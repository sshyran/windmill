/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use crate::{
    db::{UserDB, DB},
    users::Authed,
};
use axum::{
    extract::{Extension, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sql_builder::{bind::Bind, SqlBuilder};
use sqlx::FromRow;
use windmill_audit::{audit_log, ActionKind};
use windmill_common::{
    error::{Error, JsonResult, Result},
    utils::{not_found_if_none, paginate, StripPath},
};

pub fn workspaced_service() -> Router {
    Router::new()
        .route("/list", get(list_resources))
        .route("/get/*path", get(get_resource))
        .route("/exists/*path", get(exists_resource))
        .route("/get_value/*path", get(get_resource_value))
        .route("/update/*path", post(update_resource))
        .route("/delete/*path", delete(delete_resource))
        .route("/create", post(create_resource))
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct ResourceType {
    pub workspace_id: String,
    pub name: String,
    pub schema: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateResourceType {
    pub name: String,
    pub schema: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct EditResourceType {
    pub schema: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct App {
    pub workspace_id: String,
    pub path: String,
    pub value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub resource_type: String,
    pub extra_perms: serde_json::Value,
    pub is_oauth: bool,
}

#[derive(Deserialize)]
pub struct CreateResource {
    pub path: String,
    pub value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub resource_type: String,
    pub is_oauth: Option<bool>,
}
#[derive(Deserialize)]
struct EditResource {
    path: Option<String>,
    description: Option<String>,
    value: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ListResourceQuery {
    resource_type: Option<String>,
}
async fn list_resources(
    authed: Authed,
    Query(lq): Query<ListResourceQuery>,
    Query(pagination): Query<Pagination>,
    Extension(user_db): Extension<UserDB>,
    Path(w_id): Path<String>,
) -> JsonResult<Vec<Resource>> {
    let (per_page, offset) = paginate(pagination);

    let mut sqlb = SqlBuilder::select_from("resource")
        .fields(&[
            "workspace_id",
            "path",
            "null::JSONB as value",
            "description",
            "resource_type",
            "extra_perms",
            "is_oauth",
        ])
        .order_by("path", true)
        .and_where("workspace_id = ? OR workspace_id = 'starter'".bind(&w_id))
        .offset(offset)
        .limit(per_page)
        .clone();
    if let Some(rt) = &lq.resource_type {
        sqlb.and_where_eq("resource_type", "?".bind(rt));
    }

    let sql = sqlb.sql().map_err(|e| Error::InternalErr(e.to_string()))?;
    let mut tx = user_db.begin(&authed).await?;
    let rows = sqlx::query_as::<_, Resource>(&sql)
        .fetch_all(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(rows))
}

async fn get_resource(
    authed: Authed,
    Extension(user_db): Extension<UserDB>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<Resource> {
    let path = path.to_path();
    let mut tx = user_db.begin(&authed).await?;

    let resource_o = sqlx::query_as!(
        Resource,
        "SELECT * from resource WHERE path = $1 AND (workspace_id = $2 OR workspace_id = \
         'starter')",
        path.to_owned(),
        &w_id
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;

    let resource = not_found_if_none(resource_o, "Resource", path)?;
    Ok(Json(resource))
}

async fn exists_resource(
    Extension(db): Extension<DB>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<bool> {
    let path = path.to_path();

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM resource WHERE path = $1 AND workspace_id = $2)",
        path,
        w_id
    )
    .fetch_one(&db)
    .await?
    .unwrap_or(false);

    Ok(Json(exists))
}

async fn get_resource_value(
    authed: Authed,
    Extension(user_db): Extension<UserDB>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> JsonResult<Option<serde_json::Value>> {
    let path = path.to_path();
    let mut tx = user_db.begin(&authed).await?;

    let value_o = sqlx::query_scalar!(
        "SELECT value from resource WHERE path = $1 AND (workspace_id = $2 OR workspace_id = \
         'starter')",
        path.to_owned(),
        &w_id
    )
    .fetch_optional(&mut tx)
    .await?;
    tx.commit().await?;

    let value = not_found_if_none(value_o, "Resource", path)?;
    Ok(Json(value))
}

async fn create_resource(
    authed: Authed,
    Extension(user_db): Extension<UserDB>,
    Path(w_id): Path<String>,
    Json(resource): Json<CreateResource>,
) -> Result<(StatusCode, String)> {
    let mut tx = user_db.begin(&authed).await?;

    sqlx::query!(
        "INSERT INTO resource
            (workspace_id, path, value, description, resource_type, is_oauth)
            VALUES ($1, $2, $3, $4, $5, $6)",
        w_id,
        resource.path,
        resource.value,
        resource.description,
        resource.resource_type,
        resource.is_oauth.unwrap_or(false)
    )
    .execute(&mut tx)
    .await?;
    audit_log(
        &mut tx,
        &authed.username,
        "resources.create",
        ActionKind::Create,
        &w_id,
        Some(&resource.path),
        None,
    )
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        format!("resource {} created", resource.path),
    ))
}

async fn delete_resource(
    authed: Authed,
    Extension(user_db): Extension<UserDB>,
    Path((w_id, path)): Path<(String, StripPath)>,
) -> Result<String> {
    let path = path.to_path();
    let mut tx = user_db.begin(&authed).await?;

    sqlx::query!(
        "DELETE FROM resource WHERE path = $1 AND workspace_id = $2",
        path,
        w_id
    )
    .execute(&mut tx)
    .await?;
    audit_log(
        &mut tx,
        &authed.username,
        "resources.delete",
        ActionKind::Delete,
        &w_id,
        Some(path),
        None,
    )
    .await?;
    tx.commit().await?;

    Ok(format!("resource {} deleted", path))
}

async fn update_resource(
    authed: Authed,
    Extension(user_db): Extension<UserDB>,
    Path((w_id, path)): Path<(String, StripPath)>,
    Json(ns): Json<EditResource>,
) -> Result<String> {
    use sql_builder::prelude::*;

    let path = path.to_path();

    let mut sqlb = SqlBuilder::update_table("resource");
    sqlb.and_where_eq("path", "?".bind(&path));
    sqlb.and_where_eq("workspace_id", "?".bind(&w_id));

    if let Some(npath) = &ns.path {
        sqlb.set_str("path", npath);
    }
    if let Some(nvalue) = ns.value {
        sqlb.set_str("value", nvalue.to_string());
    }
    if let Some(ndesc) = ns.description {
        sqlb.set_str("description", ndesc);
    }

    sqlb.returning("path");

    let mut tx = user_db.begin(&authed).await?;

    let sql = sqlb.sql().map_err(|e| Error::InternalErr(e.to_string()))?;
    let npath_o: Option<String> = sqlx::query_scalar(&sql).fetch_optional(&mut tx).await?;

    let npath = not_found_if_none(npath_o, "Resource", path)?;

    audit_log(
        &mut tx,
        &authed.username,
        "resources.update",
        ActionKind::Update,
        &w_id,
        Some(path),
        None,
    )
    .await?;
    tx.commit().await?;

    Ok(format!("resource {} updated (npath: {:?})", path, npath))
}
