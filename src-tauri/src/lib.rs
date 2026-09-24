use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, WebviewWindow};
use uuid::Uuid;

const CATALOG_DATABASE: &str = "orbit.db";
const PROJECT_DATABASE: &str = "project.db";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalProfileInitialization {
    profile_id: String,
    workspace_id: String,
    workspace_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalProfile {
    id: String, display_name: String, local_handle: Option<String>, bio: String, location: String,
    website_url: Option<String>, github_url: Option<String>, avatar_asset_id: Option<String>, cover_asset_id: Option<String>, technologies: Vec<String>, updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateLocalProfile {
    display_name: String, local_handle: Option<String>, bio: String, location: String,
    website_url: Option<String>, github_url: Option<String>, technologies: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplaceProfileAsset { kind: String, original_name: String, bytes: Vec<u8> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileAssetBinary { media_type: String, bytes: Vec<u8> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileStat { label: String, value: i64, detail: String }

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProfileResource { id: String, project_id: Option<String>, title: String, subtitle: String, kind: String, occurred_at: i64 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileOverview { profile: LocalProfile, stats: Vec<ProfileStat>, recent_projects: Vec<LocalProjectSummary>, activity: Vec<ProfileResource>, unavailable_project_count: i64 }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileResourceQuery { tab: String, cursor: Option<String>, limit: Option<usize> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileResourcePage { items: Vec<ProfileResource>, next_cursor: Option<String>, unavailable_project_count: i64 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalProjectSummary {
    id: String,
    workspace_id: String,
    name: String,
    storage_mode: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PersonalLibraryItem {
    id: String,
    source_key: String,
    is_favorite: bool,
    added_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasSummary {
    id: String,
    project_id: String,
    name: String,
    viewport: Value,
    created_at: i64,
    updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasPort {
    id: String,
    component_id: String,
    key: String,
    direction: String,
    protocol: String,
    data: Value,
    order: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasComponent {
    id: String,
    canvas_id: String,
    component_type: String,
    label: String,
    description: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: String,
    data: Value,
    ports: Vec<CanvasPort>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasConnectionSummary {
    id: String,
    canvas_id: String,
    source_component_id: String,
    source_port_id: String,
    target_component_id: String,
    target_port_id: String,
    connection_type: String,
    data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasVisual {
    id: String, canvas_id: String, visual_type: String, x: f64, y: f64, width: f64, height: f64, data: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasSnapshot {
    canvas: CanvasSummary,
    components: Vec<CanvasComponent>,
    connections: Vec<CanvasConnectionSummary>,
    visuals: Vec<CanvasVisual>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCanvasVisual { project_id: String, canvas_id: String, visual_type: String, x: f64, y: f64, width: f64, height: f64, data: Value }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCanvasVisual { project_id: String, canvas_id: String, visual_id: String, data: Value }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteCanvasVisual { project_id: String, canvas_id: String, visual_id: String }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteSummary {
    id: String, project_id: String, title: String, content: String, content_format: String,
    is_pinned: bool, is_archived: bool, tags: Vec<String>, created_at: i64, updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocSummary {
    id: String, project_id: String, space_id: Option<String>, parent_id: Option<String>, kind: String,
    title: String, slug: String, content: String, content_format: String, sort_order: i64,
    origin: String, created_at: i64, updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDoc { project_id: String, title: Option<String>, parent_id: Option<String>, kind: Option<String> }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateDoc { project_id: String, document_id: String, title: String, content: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentAction { project_id: String, document_id: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportDocMarkdown { project_id: String, document_id: String, destination_path: String }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiSettings { enabled: bool, provider: String, model: String, has_openai_api_key: bool, has_github_token: bool }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveAiSettings { enabled: bool, model: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveSecret { value: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartArchitectureAnalysis { project_id: String, repository_url: Option<String>, document_path: Option<String>, folder_path: Option<String>, consent_to_send_sources: bool }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArchitectureGeneration { id: String, project_id: String, status: String, source_mode: String, sources: Value, architecture_model: Option<Value>, documentation_markdown: Option<String>, error: Option<String>, created_at: i64, updated_at: i64 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEntry {
    id: String, project_id: String, entry_date: String, content: String, created_at: i64, updated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveSimulationRun {
    project_id: String,
    canvas_id: String,
    scenario: String,
    seed: i64,
    config: Value,
    canvas_snapshot: Value,
    result: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SimulationRunSummary { id: String, created_at: i64 }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCalendarEntry { project_id: String, entry_date: String, content: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateNote { project_id: String, title: Option<String> }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateNote { project_id: String, note_id: String, title: String, content: String, tags: Vec<String> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetSummary { id: String, project_id: String, original_name: String, stored_path: String, media_type: String, byte_size: i64, created_at: i64 }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAsset { project_id: String, original_name: String, media_type: String, bytes: Vec<u8> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetBinary { media_type: String, bytes: Vec<u8> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateCatalogState { favorite_template_ids: Vec<String> }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateProjectResult { project_id: String, canvas_id: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplatePortSeed {
    key: String,
    direction: String,
    protocol: String,
    #[serde(default)]
    data: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateComponentSeed {
    component_type: String, label: String, description: String, x: f64, y: f64,
    width: f64, height: f64, color: String, ports: Vec<TemplatePortSeed>,
    #[serde(default)] data: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateConnectionSeed { source_index: usize, source_port_key: String, target_index: usize, target_port_key: String, connection_type: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateNoteSeed { title: String, content: String, #[serde(default)] tags: Vec<String> }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateDocSeed { title: String, content: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemplateSeed {
    version: String, canvas_name: String, components: Vec<TemplateComponentSeed>,
    #[serde(default)] connections: Vec<TemplateConnectionSeed>,
    #[serde(default)] notes: Vec<TemplateNoteSeed>, #[serde(default)] docs: Vec<TemplateDocSeed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyTemplateToProject { template_id: String, project_id: String, seed: TemplateSeed }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateProjectFromTemplate { template_id: String, project_name: String, seed: TemplateSeed }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadAsset { project_id: String, asset_id: String }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewPort {
    key: String,
    direction: String,
    protocol: String,
    #[serde(default)]
    data: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCanvasComponent {
    project_id: String,
    canvas_id: String,
    component_type: String,
    label: String,
    description: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: String,
    #[serde(default)]
    data: Value,
    ports: Vec<NewPort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveCanvasElement {
    project_id: String,
    canvas_id: String,
    element_id: String,
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteCanvasComponent {
    project_id: String,
    canvas_id: String,
    component_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCanvasComponent {
    project_id: String,
    canvas_id: String,
    component_id: String,
    label: String,
    description: String,
    color: String,
    #[serde(default)]
    data: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCanvasConnection {
    project_id: String,
    canvas_id: String,
    source_component_id: String,
    source_port_id: String,
    target_component_id: String,
    target_port_id: String,
    connection_type: String,
    #[serde(default)]
    data: Value,
}

fn timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .map_err(|error| format!("Não foi possível gerar o horário atual: {error}"))
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Não foi possível localizar os dados locais do Orbit: {error}"))
}

fn catalog_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join(CATALOG_DATABASE))
}

fn open_catalog(app: &AppHandle) -> Result<Connection, String> {
    let directory = data_dir(app)?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Não foi possível criar os dados locais do Orbit: {error}"))?;
    let connection = Connection::open(catalog_path(app)?)
        .map_err(|error| format!("Não foi possível abrir o catálogo local: {error}"))?;
    migrate_catalog(&connection)?;
    Ok(connection)
}

fn migrate_catalog(connection: &Connection) -> Result<(), String> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);",
    ).map_err(|error| format!("Não foi possível preparar as migrations do catálogo: {error}"))?;
    let applied = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 1)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("Não foi possível verificar a migration do catálogo: {error}"))?;
    if !applied {
        let now = timestamp()?;
        let transaction = connection.unchecked_transaction().map_err(|error| {
            format!("Não foi possível iniciar a migration do catálogo: {error}")
        })?;
        transaction.execute_batch(
            "CREATE TABLE local_profiles (id TEXT PRIMARY KEY NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
             CREATE TABLE workspaces (id TEXT PRIMARY KEY NOT NULL, profile_id TEXT NOT NULL REFERENCES local_profiles(id), name TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER);
             CREATE TABLE projects (id TEXT PRIMARY KEY NOT NULL, workspace_id TEXT NOT NULL REFERENCES workspaces(id), name TEXT NOT NULL, storage_mode TEXT NOT NULL CHECK(storage_mode IN ('local','cloud')), relative_path TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER);
             CREATE INDEX projects_by_workspace ON projects(workspace_id, deleted_at, updated_at DESC);",
        ).map_err(|error| format!("Não foi possível aplicar a migration do catálogo: {error}"))?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(1, ?1)",
                [now],
            )
            .map_err(|error| {
                format!("Não foi possível registrar a migration do catálogo: {error}")
            })?;
        transaction.commit().map_err(|error| {
            format!("Não foi possível confirmar a migration do catálogo: {error}")
        })?;
    }
    migrate_component_library_catalog(connection)?;
    migrate_template_catalog(connection)?;
    migrate_ai_catalog(connection)?;
    migrate_profile_catalog(connection)
}

fn migrate_component_library_catalog(connection: &Connection) -> Result<(), String> {
    let applied = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("Não foi possível verificar a migration da biblioteca de Components: {error}"))?;
    if applied {
        return Ok(());
    }
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration da biblioteca de Components: {error}"))?;
    transaction.execute_batch(
        "CREATE TABLE component_libraries (
            id TEXT PRIMARY KEY NOT NULL,
            profile_id TEXT NOT NULL REFERENCES local_profiles(id),
            label TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
            sort_order INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(profile_id, id)
        );
        CREATE TABLE library_components (
            id TEXT PRIMARY KEY NOT NULL,
            profile_id TEXT NOT NULL REFERENCES local_profiles(id),
            source_key TEXT NOT NULL,
            added_at INTEGER NOT NULL,
            removed_at INTEGER,
            UNIQUE(profile_id, source_key)
        );
        CREATE TABLE component_favorites (
            library_component_id TEXT PRIMARY KEY NOT NULL REFERENCES library_components(id),
            created_at INTEGER NOT NULL
        );
        CREATE INDEX library_components_by_profile ON library_components(profile_id, removed_at, added_at);
        CREATE TABLE custom_component_definitions (
            id TEXT PRIMARY KEY NOT NULL,
            profile_id TEXT NOT NULL REFERENCES local_profiles(id),
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            category TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            color TEXT NOT NULL,
            icon_library_id TEXT NOT NULL,
            icon_key TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            deleted_at INTEGER
        );",
    ).map_err(|error| format!("Não foi possível criar as tabelas da biblioteca de Components: {error}"))?;
    transaction.execute(
        "INSERT INTO schema_migrations(version, applied_at) VALUES(4, ?1)",
        [now],
    ).map_err(|error| format!("Não foi possível registrar a migration da biblioteca de Components: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration da biblioteca de Components: {error}"))
}

fn migrate_template_catalog(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 5)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration de Templates: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration de Templates: {error}"))?;
    transaction.execute_batch("CREATE TABLE template_favorites (profile_id TEXT NOT NULL REFERENCES local_profiles(id), template_id TEXT NOT NULL, created_at INTEGER NOT NULL, PRIMARY KEY(profile_id, template_id));")
        .map_err(|error| format!("Não foi possível criar os favoritos de Templates: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(5, ?1)", [now])
        .map_err(|error| format!("Não foi possível registrar a migration de Templates: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration de Templates: {error}"))
}

fn migrate_ai_catalog(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 6)", [], |row| row.get::<_, bool>(0)).map_err(|error| format!("Não foi possível verificar a migration de IA: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    connection.execute_batch("CREATE TABLE ai_settings (id INTEGER PRIMARY KEY CHECK(id = 1), enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)), provider TEXT NOT NULL DEFAULT 'openai', model TEXT NOT NULL DEFAULT 'gpt-4o-mini', updated_at INTEGER NOT NULL);")
        .map_err(|error| format!("Não foi possível criar configurações de IA: {error}"))?;
    connection.execute("INSERT INTO ai_settings(id, enabled, provider, model, updated_at) VALUES(1, 0, 'openai', 'gpt-4o-mini', ?1)", [now]).map_err(|error| format!("Não foi possível iniciar configurações de IA: {error}"))?;
    connection.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(6, ?1)", [now]).map_err(|error| format!("Não foi possível registrar a migration de IA: {error}"))?;
    Ok(())
}

fn migrate_profile_catalog(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 7)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration de Profile: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a migration de Profile: {error}"))?;
    transaction.execute_batch("CREATE TABLE profile_details (profile_id TEXT PRIMARY KEY NOT NULL REFERENCES local_profiles(id), display_name TEXT NOT NULL DEFAULT 'Orbit User' CHECK(length(display_name) BETWEEN 1 AND 80), local_handle TEXT, bio TEXT NOT NULL DEFAULT '' CHECK(length(bio) <= 280), location TEXT NOT NULL DEFAULT '' CHECK(length(location) <= 80), website_url TEXT, github_url TEXT, avatar_asset_id TEXT, cover_asset_id TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL); CREATE UNIQUE INDEX profile_details_handle_unique ON profile_details(local_handle COLLATE NOCASE) WHERE local_handle IS NOT NULL; CREATE TABLE profile_assets (id TEXT PRIMARY KEY NOT NULL, profile_id TEXT NOT NULL REFERENCES local_profiles(id), kind TEXT NOT NULL CHECK(kind IN ('avatar','cover')), stored_path TEXT NOT NULL UNIQUE, media_type TEXT NOT NULL, byte_size INTEGER NOT NULL CHECK(byte_size >= 0), created_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX profile_assets_by_profile ON profile_assets(profile_id, kind, deleted_at); CREATE TABLE profile_technologies (profile_id TEXT NOT NULL REFERENCES local_profiles(id), slug TEXT NOT NULL, label TEXT NOT NULL CHECK(length(label) BETWEEN 1 AND 32), sort_order INTEGER NOT NULL, created_at INTEGER NOT NULL, PRIMARY KEY(profile_id, slug));")
        .map_err(|error| format!("Não foi possível criar a estrutura de Profile: {error}"))?;
    transaction.execute("INSERT INTO profile_details(profile_id, created_at, updated_at) SELECT id, ?1, ?1 FROM local_profiles", [now]).map_err(|error| format!("Não foi possível iniciar os perfis existentes: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(7, ?1)", [now]).map_err(|error| format!("Não foi possível registrar a migration de Profile: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration de Profile: {error}"))
}

fn ensure_component_library(connection: &Connection, profile_id: &str) -> Result<(), String> {
    let now = timestamp()?;
    connection.execute(
        "INSERT OR IGNORE INTO component_libraries(id, profile_id, label, enabled, sort_order, created_at, updated_at)
         VALUES('orbit-core', ?1, 'Orbit Core', 1, 0, ?2, ?2)",
        params![profile_id, now],
    ).map_err(|error| format!("Não foi possível preparar a biblioteca Orbit Core: {error}"))?;
    connection.execute(
        "INSERT OR IGNORE INTO component_libraries(id, profile_id, label, enabled, sort_order, created_at, updated_at)
         VALUES('aws', ?1, 'AWS Architecture Icons', 0, 1, ?2, ?2)",
        params![profile_id, now],
    ).map_err(|error| format!("Não foi possível preparar a biblioteca AWS: {error}"))?;
    for source_key in ["client", "api-server", "database", "cache", "load-balancer"] {
        connection.execute(
            "INSERT OR IGNORE INTO library_components(id, profile_id, source_key, added_at) VALUES(?1, ?2, ?3, ?4)",
            params![Uuid::now_v7().to_string(), profile_id, source_key, now],
        ).map_err(|error| format!("Não foi possível preparar o Component base {source_key}: {error}"))?;
    }
    Ok(())
}

fn enable_aws_library(connection: &Connection, profile_id: &str) -> Result<(), String> {
    ensure_component_library(connection, profile_id)?;
    connection.execute("UPDATE component_libraries SET enabled = 1, updated_at = ?1 WHERE id = 'aws' AND profile_id = ?2", params![timestamp()?, profile_id])
        .map_err(|error| format!("Não foi possível habilitar a biblioteca AWS: {error}"))?;
    Ok(())
}

fn migrate_project_database(path: &Path) -> Result<(), String> {
    let connection = Connection::open(path)
        .map_err(|error| format!("Não foi possível abrir o banco do Project: {error}"))?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);",
    ).map_err(|error| format!("Não foi possível preparar as migrations do Project: {error}"))?;
    let applied = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 1)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("Não foi possível verificar a migration do Project: {error}"))?;
    if !applied {
        let now = timestamp()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|error| format!("Não foi possível iniciar a migration do Project: {error}"))?;
        transaction.execute_batch(
            "CREATE TABLE project_metadata (project_id TEXT PRIMARY KEY NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
             CREATE TABLE assets (id TEXT PRIMARY KEY NOT NULL, original_name TEXT NOT NULL, stored_path TEXT NOT NULL UNIQUE, media_type TEXT NOT NULL, byte_size INTEGER NOT NULL CHECK(byte_size >= 0), created_at INTEGER NOT NULL, deleted_at INTEGER);
             CREATE TABLE revisions (id TEXT PRIMARY KEY NOT NULL, entity_id TEXT NOT NULL, entity_type TEXT NOT NULL, operation TEXT NOT NULL, payload TEXT NOT NULL, created_at INTEGER NOT NULL);
             CREATE INDEX revisions_by_entity ON revisions(entity_id, created_at DESC);",
        ).map_err(|error| format!("Não foi possível aplicar a migration do Project: {error}"))?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES(1, ?1)",
                [now],
            )
            .map_err(|error| {
                format!("Não foi possível registrar a migration do Project: {error}")
            })?;
        transaction.commit().map_err(|error| {
            format!("Não foi possível confirmar a migration do Project: {error}")
        })?;
    }
    migrate_canvas_database(&connection)?;
    migrate_notes_database(&connection)?;
    migrate_template_project_database(&connection)?;
    migrate_calendar_project_database(&connection)?;
    migrate_docs_project_database(&connection)?;
    migrate_ai_project_database(&connection)?;
    migrate_simulation_project_database(&connection)
}

fn migrate_notes_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 3)", [], |row| row.get::<_, bool>(0)).map_err(|error| format!("Não foi possível verificar a migration das Notes: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a migration das Notes: {error}"))?;
    transaction.execute_batch("CREATE TABLE notes (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, title TEXT NOT NULL, content TEXT NOT NULL, content_format TEXT NOT NULL DEFAULT 'markdown', tags TEXT NOT NULL DEFAULT '[]', is_pinned INTEGER NOT NULL DEFAULT 0 CHECK(is_pinned IN (0,1)), is_archived INTEGER NOT NULL DEFAULT 0 CHECK(is_archived IN (0,1)), created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX notes_by_project ON notes(project_id, deleted_at, is_archived, is_pinned DESC, updated_at DESC);").map_err(|error| format!("Não foi possível criar as tabelas das Notes: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(3, ?1)", [now]).map_err(|error| format!("Não foi possível registrar a migration das Notes: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration das Notes: {error}"))
}

fn migrate_template_project_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration de conteúdo de Templates: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration de conteúdo de Templates: {error}"))?;
    transaction.execute_batch("ALTER TABLE project_metadata ADD COLUMN template_id TEXT; ALTER TABLE project_metadata ADD COLUMN template_version TEXT; CREATE TABLE docs (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, title TEXT NOT NULL, content TEXT NOT NULL, content_format TEXT NOT NULL DEFAULT 'markdown', created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX docs_by_project ON docs(project_id, deleted_at, updated_at DESC);")
        .map_err(|error| format!("Não foi possível criar a estrutura de Templates do Project: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(4, ?1)", [now])
        .map_err(|error| format!("Não foi possível registrar a migration de conteúdo de Templates: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration de conteúdo de Templates: {error}"))
}

fn migrate_calendar_project_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 5)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration do Calendar: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration do Calendar: {error}"))?;
    transaction.execute_batch("CREATE TABLE calendar_entries (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, entry_date TEXT NOT NULL, content TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX calendar_entries_by_project_date ON calendar_entries(project_id, entry_date, deleted_at);")
        .map_err(|error| format!("Não foi possível criar a estrutura do Calendar: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(5, ?1)", [now])
        .map_err(|error| format!("Não foi possível registrar a migration do Calendar: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration do Calendar: {error}"))
}

fn migrate_docs_project_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 6)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration dos Docs: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration dos Docs: {error}"))?;
    transaction.execute_batch("CREATE TABLE doc_spaces (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, name TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX doc_spaces_by_project ON doc_spaces(project_id, deleted_at, sort_order, updated_at); ALTER TABLE docs ADD COLUMN space_id TEXT; ALTER TABLE docs ADD COLUMN parent_id TEXT; ALTER TABLE docs ADD COLUMN kind TEXT NOT NULL DEFAULT 'page'; ALTER TABLE docs ADD COLUMN slug TEXT NOT NULL DEFAULT ''; ALTER TABLE docs ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0; ALTER TABLE docs ADD COLUMN origin TEXT NOT NULL DEFAULT 'manual'; CREATE INDEX docs_tree_by_project ON docs(project_id, space_id, parent_id, deleted_at, sort_order, updated_at DESC);")
        .map_err(|error| format!("Não foi possível criar a estrutura dos Docs: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(6, ?1)", [now])
        .map_err(|error| format!("Não foi possível registrar a migration dos Docs: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration dos Docs: {error}"))
}

fn migrate_ai_project_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 7)", [], |row| row.get::<_, bool>(0)).map_err(|error| format!("Não foi possível verificar a migration de gerações: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    connection.execute_batch("CREATE TABLE ai_generations (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, status TEXT NOT NULL, source_mode TEXT NOT NULL, sources TEXT NOT NULL, architecture_model TEXT, documentation_markdown TEXT, error TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER); CREATE INDEX ai_generations_by_project ON ai_generations(project_id, deleted_at, updated_at DESC);")
        .map_err(|error| format!("Não foi possível criar as gerações de IA: {error}"))?;
    connection.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(7, ?1)", [now]).map_err(|error| format!("Não foi possível registrar a migration de gerações: {error}"))?;
    Ok(())
}

fn migrate_simulation_project_database(connection: &Connection) -> Result<(), String> {
    let applied = connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 8)", [], |row| row.get::<_, bool>(0))
        .map_err(|error| format!("Não foi possível verificar a migration das simulações: {error}"))?;
    if applied { return Ok(()); }
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration das simulações: {error}"))?;
    transaction.execute_batch("CREATE TABLE simulation_runs (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, canvas_id TEXT NOT NULL, scenario TEXT NOT NULL, seed INTEGER NOT NULL, config TEXT NOT NULL, canvas_snapshot TEXT NOT NULL, result TEXT NOT NULL, created_at INTEGER NOT NULL); CREATE INDEX simulation_runs_by_project ON simulation_runs(project_id, canvas_id, created_at DESC);")
        .map_err(|error| format!("Não foi possível criar a estrutura das simulações: {error}"))?;
    transaction.execute("INSERT INTO schema_migrations(version, applied_at) VALUES(8, ?1)", [now])
        .map_err(|error| format!("Não foi possível registrar a migration das simulações: {error}"))?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a migration das simulações: {error}"))
}

fn migrate_canvas_database(connection: &Connection) -> Result<(), String> {
    let applied = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("Não foi possível verificar a migration do Canvas: {error}"))?;
    if applied {
        return Ok(());
    }
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a migration do Canvas: {error}"))?;
    transaction.execute_batch("CREATE TABLE canvases (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL, name TEXT NOT NULL, viewport TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER);
CREATE TABLE canvas_elements (id TEXT PRIMARY KEY NOT NULL, canvas_id TEXT NOT NULL REFERENCES canvases(id), kind TEXT NOT NULL CHECK(kind IN ('visual','component')), type TEXT NOT NULL, x REAL NOT NULL, y REAL NOT NULL, width REAL NOT NULL CHECK(width >= 0), height REAL NOT NULL CHECK(height >= 0), parent_id TEXT REFERENCES canvas_elements(id), data TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER);
CREATE TABLE components (element_id TEXT PRIMARY KEY NOT NULL REFERENCES canvas_elements(id), label TEXT NOT NULL, description TEXT NOT NULL, color TEXT NOT NULL, icon TEXT NOT NULL, properties TEXT NOT NULL);
CREATE TABLE component_ports (id TEXT PRIMARY KEY NOT NULL, component_id TEXT NOT NULL REFERENCES components(element_id), key TEXT NOT NULL, direction TEXT NOT NULL CHECK(direction IN ('input','output','bidirectional')), protocol TEXT NOT NULL CHECK(protocol IN ('http','sql','data','event')), data TEXT NOT NULL, order_index INTEGER NOT NULL, UNIQUE(component_id,key));
CREATE TABLE connections (id TEXT PRIMARY KEY NOT NULL, canvas_id TEXT NOT NULL REFERENCES canvases(id), source_component_id TEXT NOT NULL REFERENCES components(element_id), source_port_id TEXT NOT NULL REFERENCES component_ports(id), target_component_id TEXT NOT NULL REFERENCES components(element_id), target_port_id TEXT NOT NULL REFERENCES component_ports(id), type TEXT NOT NULL, data TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER, CHECK(source_component_id <> target_component_id));
CREATE INDEX canvases_by_project ON canvases(project_id, deleted_at, updated_at DESC); CREATE INDEX elements_by_canvas ON canvas_elements(canvas_id, deleted_at, updated_at); CREATE INDEX ports_by_component ON component_ports(component_id, order_index); CREATE INDEX connections_by_canvas ON connections(canvas_id, deleted_at);").map_err(|error| format!("Não foi possível criar as tabelas do Canvas: {error}"))?;
    transaction
        .execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES(2, ?1)",
            [now],
        )
        .map_err(|error| format!("Não foi possível registrar a migration do Canvas: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar a migration do Canvas: {error}"))
}

fn open_project_database(app: &AppHandle, project_id: &str) -> Result<Connection, String> {
    let catalog = open_catalog(app)?;
    let relative_path: String = catalog
        .query_row(
            "SELECT relative_path FROM projects WHERE id = ?1 AND deleted_at IS NULL",
            [project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Não foi possível localizar o Project: {error}"))?
        .ok_or_else(|| "O Project solicitado não existe ou foi removido.".to_string())?;
    let path = data_dir(app)?.join(relative_path).join(PROJECT_DATABASE);
    migrate_project_database(&path)?;
    let connection = Connection::open(path)
        .map_err(|error| format!("Não foi possível abrir o banco do Project: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("Não foi possível preparar o banco do Project: {error}"))?;
    Ok(connection)
}

fn parse_json(value: String, field: &str) -> Result<Value, String> {
    serde_json::from_str(&value)
        .map_err(|error| format!("Não foi possível ler {field} do Canvas: {error}"))
}

fn append_revision(
    transaction: &rusqlite::Transaction<'_>,
    entity_id: &str,
    entity_type: &str,
    operation: &str,
    payload: Value,
    now: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO revisions(id, entity_id, entity_type, operation, payload, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![Uuid::now_v7().to_string(), entity_id, entity_type, operation, payload.to_string(), now],
        )
        .map_err(|error| format!("Não foi possível registrar a revision do Canvas: {error}"))?;
    Ok(())
}

fn ensure_canvas(connection: &mut Connection, project_id: &str) -> Result<CanvasSummary, String> {
    let existing = connection
        .query_row(
            "SELECT id, project_id, name, viewport, created_at, updated_at FROM canvases WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY created_at LIMIT 1",
            [project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?)),
        )
        .optional()
        .map_err(|error| format!("Não foi possível ler o Canvas: {error}"))?;
    if let Some((id, project_id, name, viewport, created_at, updated_at)) = existing {
        return Ok(CanvasSummary {
            id,
            project_id,
            name,
            viewport: parse_json(viewport, "viewport")?,
            created_at,
            updated_at,
        });
    }

    let id = Uuid::now_v7().to_string();
    let now = timestamp()?;
    let viewport = json!({ "x": 0.0, "y": 0.0, "zoom": 1.0 });
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar o Canvas inicial: {error}"))?;
    transaction
        .execute(
            "INSERT INTO canvases(id, project_id, name, viewport, created_at, updated_at) VALUES(?1, ?2, 'Untitled Canvas', ?3, ?4, ?4)",
            params![id, project_id, viewport.to_string(), now],
        )
        .map_err(|error| format!("Não foi possível criar o Canvas inicial: {error}"))?;
    append_revision(
        &transaction,
        &id,
        "canvas",
        "created",
        json!({ "name": "Untitled Canvas" }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar o Canvas inicial: {error}"))?;
    Ok(CanvasSummary {
        id,
        project_id: project_id.to_string(),
        name: "Untitled Canvas".to_string(),
        viewport,
        created_at: now,
        updated_at: now,
    })
}

fn assert_canvas(connection: &Connection, canvas_id: &str, project_id: &str) -> Result<(), String> {
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM canvases WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL)",
            params![canvas_id, project_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("Não foi possível validar o Canvas: {error}"))?;
    if exists {
        Ok(())
    } else {
        Err("O Canvas não pertence ao Project informado.".to_string())
    }
}

fn load_canvas_snapshot(
    connection: &mut Connection,
    project_id: &str,
    requested_canvas_id: Option<&str>,
) -> Result<CanvasSnapshot, String> {
    let canvas = match requested_canvas_id {
        Some(canvas_id) => {
            let selected = connection.query_row(
                "SELECT id, project_id, name, viewport, created_at, updated_at FROM canvases WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL",
                params![canvas_id, project_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, i64>(5)?)),
            ).optional().map_err(|error| format!("Não foi possível carregar o Canvas solicitado: {error}"))?
                .ok_or_else(|| "O Canvas solicitado não pertence ao Project informado.".to_string())?;
            CanvasSummary { id: selected.0, project_id: selected.1, name: selected.2, viewport: parse_json(selected.3, "viewport")?, created_at: selected.4, updated_at: selected.5 }
        }
        None => ensure_canvas(connection, project_id)?,
    };
    let mut components_statement = connection
        .prepare("SELECT e.id, e.canvas_id, e.type, c.label, c.description, e.x, e.y, e.width, e.height, c.color, e.data FROM canvas_elements e JOIN components c ON c.element_id = e.id WHERE e.canvas_id = ?1 AND e.kind = 'component' AND e.deleted_at IS NULL ORDER BY e.created_at")
        .map_err(|error| format!("Não foi possível preparar os Components: {error}"))?;
    let rows = components_statement
        .query_map([&canvas.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, f64>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, f64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .map_err(|error| format!("Não foi possível carregar os Components: {error}"))?;
    let mut components = Vec::new();
    for row in rows {
        let (id, canvas_id, component_type, label, description, x, y, width, height, color, data) =
            row.map_err(|error| format!("Não foi possível ler um Component: {error}"))?;
        let mut ports_statement = connection
            .prepare("SELECT id, component_id, key, direction, protocol, data, order_index FROM component_ports WHERE component_id = ?1 ORDER BY order_index")
            .map_err(|error| format!("Não foi possível preparar os Ports: {error}"))?;
        let port_rows = ports_statement
            .query_map([&id], |port| {
                Ok((
                    port.get::<_, String>(0)?,
                    port.get::<_, String>(1)?,
                    port.get::<_, String>(2)?,
                    port.get::<_, String>(3)?,
                    port.get::<_, String>(4)?,
                    port.get::<_, String>(5)?,
                    port.get::<_, i64>(6)?,
                ))
            })
            .map_err(|error| format!("Não foi possível carregar os Ports: {error}"))?;
        let mut ports = Vec::new();
        for port in port_rows {
            let (id, component_id, key, direction, protocol, data, order) =
                port.map_err(|error| format!("Não foi possível ler um Port: {error}"))?;
            ports.push(CanvasPort {
                id,
                component_id,
                key,
                direction,
                protocol,
                data: parse_json(data, "dados do Port")?,
                order,
            });
        }
        components.push(CanvasComponent {
            id,
            canvas_id,
            component_type,
            label,
            description,
            x,
            y,
            width,
            height,
            color,
            data: parse_json(data, "dados do Component")?,
            ports,
        });
    }
    let mut connections_statement = connection
        .prepare("SELECT id, canvas_id, source_component_id, source_port_id, target_component_id, target_port_id, type, data FROM connections WHERE canvas_id = ?1 AND deleted_at IS NULL ORDER BY created_at")
        .map_err(|error| format!("Não foi possível preparar as Connections: {error}"))?;
    let connections = connections_statement
        .query_map([&canvas.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|error| format!("Não foi possível carregar as Connections: {error}"))?
        .map(|row| {
            row.map_err(|error| format!("Não foi possível ler uma Connection: {error}"))
                .and_then(
                    |(
                        id,
                        canvas_id,
                        source_component_id,
                        source_port_id,
                        target_component_id,
                        target_port_id,
                        connection_type,
                        data,
                    )| {
                        Ok(CanvasConnectionSummary {
                            id,
                            canvas_id,
                            source_component_id,
                            source_port_id,
                            target_component_id,
                            target_port_id,
                            connection_type,
                            data: parse_json(data, "dados da Connection")?,
                        })
                    },
                )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut visuals_statement = connection.prepare("SELECT id, canvas_id, type, x, y, width, height, data FROM canvas_elements WHERE canvas_id = ?1 AND kind = 'visual' AND deleted_at IS NULL ORDER BY created_at").map_err(|error| format!("Não foi possível preparar os elementos visuais: {error}"))?;
    let visuals = visuals_statement.query_map([&canvas.id], |row| Ok(CanvasVisual { id: row.get(0)?, canvas_id: row.get(1)?, visual_type: row.get(2)?, x: row.get(3)?, y: row.get(4)?, width: row.get(5)?, height: row.get(6)?, data: parse_json(row.get::<_, String>(7)?, "dados do elemento visual").unwrap_or(Value::Null) })).map_err(|error| format!("Não foi possível carregar os elementos visuais: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler um elemento visual: {error}"))?;
    Ok(CanvasSnapshot {
        canvas,
        components,
        connections,
        visuals,
    })
}

#[tauri::command]
fn initialize_local_profile(app: AppHandle) -> Result<LocalProfileInitialization, String> {
    let connection = open_catalog(&app)?;
    let existing = connection.query_row(
        "SELECT p.id, w.id, w.name FROM local_profiles p JOIN workspaces w ON w.profile_id = p.id WHERE w.deleted_at IS NULL ORDER BY w.created_at LIMIT 1",
        [], |row| Ok(LocalProfileInitialization { profile_id: row.get(0)?, workspace_id: row.get(1)?, workspace_name: row.get(2)? }),
    ).optional().map_err(|error| format!("Não foi possível ler o perfil local: {error}"))?;
    if let Some(value) = existing {
        ensure_component_library(&connection, &value.profile_id)?;
        return Ok(value);
    }
    let profile_id = Uuid::now_v7().to_string();
    let workspace_id = Uuid::now_v7().to_string();
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar o perfil local: {error}"))?;
    transaction
        .execute(
            "INSERT INTO local_profiles(id, created_at, updated_at) VALUES(?1, ?2, ?2)",
            params![profile_id, now],
        )
        .map_err(|error| format!("Não foi possível criar o perfil local: {error}"))?;
    transaction.execute("INSERT INTO workspaces(id, profile_id, name, created_at, updated_at) VALUES(?1, ?2, 'Personal Workspace', ?3, ?3)", params![workspace_id, profile_id, now]).map_err(|error| format!("Não foi possível criar o Personal Workspace: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar o perfil local: {error}"))?;
    ensure_component_library(&connection, &profile_id)?;
    Ok(LocalProfileInitialization {
        profile_id,
        workspace_id,
        workspace_name: "Personal Workspace".to_string(),
    })
}

fn local_profile_id(connection: &Connection) -> Result<String, String> {
    connection.query_row(
        "SELECT id FROM local_profiles ORDER BY created_at LIMIT 1",
        [],
        |row| row.get(0),
    ).map_err(|error| format!("Não foi possível localizar o perfil local: {error}"))
}

fn ensure_profile_details(connection: &Connection, profile_id: &str) -> Result<(), String> {
    let now = timestamp()?;
    connection.execute("INSERT OR IGNORE INTO profile_details(profile_id, created_at, updated_at) VALUES(?1, ?2, ?2)", params![profile_id, now])
        .map_err(|error| format!("Não foi possível preparar os dados do Profile: {error}"))?;
    Ok(())
}

fn normalized_technology(value: &str) -> String {
    value.trim().to_lowercase().chars().filter(|character| character.is_ascii_alphanumeric()).collect()
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|item| { let trimmed = item.trim().to_string(); (!trimmed.is_empty()).then_some(trimmed) })
}

fn valid_profile_url(value: &str, github: bool) -> bool {
    value.starts_with("https://") && (!github || value.starts_with("https://github.com/"))
}

fn read_local_profile(connection: &Connection, profile_id: &str) -> Result<LocalProfile, String> {
    ensure_profile_details(connection, profile_id)?;
    let mut profile = connection.query_row("SELECT profile_id, display_name, local_handle, bio, location, website_url, github_url, avatar_asset_id, cover_asset_id, updated_at FROM profile_details WHERE profile_id = ?1", [profile_id], |row| Ok(LocalProfile { id: row.get(0)?, display_name: row.get(1)?, local_handle: row.get(2)?, bio: row.get(3)?, location: row.get(4)?, website_url: row.get(5)?, github_url: row.get(6)?, avatar_asset_id: row.get(7)?, cover_asset_id: row.get(8)?, technologies: Vec::new(), updated_at: row.get(9)? }))
        .map_err(|error| format!("Não foi possível ler o Profile: {error}"))?;
    let mut statement = connection.prepare("SELECT label FROM profile_technologies WHERE profile_id = ?1 ORDER BY sort_order, label COLLATE NOCASE").map_err(|error| format!("Não foi possível preparar as tecnologias: {error}"))?;
    profile.technologies = statement.query_map([profile_id], |row| row.get(0)).map_err(|error| format!("Não foi possível listar as tecnologias: {error}"))?.collect::<Result<Vec<String>, _>>().map_err(|error| format!("Não foi possível ler as tecnologias: {error}"))?;
    Ok(profile)
}

#[tauri::command]
fn get_local_profile(app: AppHandle) -> Result<LocalProfile, String> {
    let initialization = initialize_local_profile(app.clone())?;
    let catalog = open_catalog(&app)?;
    read_local_profile(&catalog, &initialization.profile_id)
}

#[tauri::command]
fn update_local_profile(app: AppHandle, input: UpdateLocalProfile) -> Result<LocalProfile, String> {
    let initialization = initialize_local_profile(app.clone())?;
    let name = input.display_name.trim();
    if name.is_empty() || name.chars().count() > 80 { return Err("O nome deve ter entre 1 e 80 caracteres.".to_string()); }
    if input.bio.chars().count() > 280 || input.location.chars().count() > 80 { return Err("Biografia ou localização excedem o limite permitido.".to_string()); }
    let handle = optional_trimmed(input.local_handle);
    if handle.as_ref().is_some_and(|value| value.chars().count() < 3 || value.chars().count() > 32 || !value.chars().all(|character| character.is_ascii_alphanumeric() || character == '_')) { return Err("O identificador deve ter 3 a 32 letras, números ou sublinhados.".to_string()); }
    let website = optional_trimmed(input.website_url); let github = optional_trimmed(input.github_url);
    if website.as_ref().is_some_and(|value| !valid_profile_url(value, false)) || github.as_ref().is_some_and(|value| !valid_profile_url(value, true)) { return Err("Use links HTTPS válidos. O GitHub deve usar github.com.".to_string()); }
    if input.technologies.len() > 20 { return Err("Escolha no máximo 20 tecnologias.".to_string()); }
    let mut technologies = Vec::new();
    for value in input.technologies { let label = value.trim().to_string(); let slug = normalized_technology(&label); if label.is_empty() || label.chars().count() > 32 || slug.is_empty() { return Err("Cada tecnologia deve ter entre 1 e 32 caracteres válidos.".to_string()); } if !technologies.iter().any(|(existing, _): &(String, String)| existing == &slug) { technologies.push((slug, label)); } }
    let catalog = open_catalog(&app)?; ensure_profile_details(&catalog, &initialization.profile_id)?; let now = timestamp()?;
    let transaction = catalog.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a atualização do Profile: {error}"))?;
    transaction.execute("UPDATE profile_details SET display_name = ?1, local_handle = ?2, bio = ?3, location = ?4, website_url = ?5, github_url = ?6, updated_at = ?7 WHERE profile_id = ?8", params![name, handle, input.bio.trim(), input.location.trim(), website, github, now, initialization.profile_id]).map_err(|error| format!("Não foi possível salvar o Profile: {error}"))?;
    transaction.execute("DELETE FROM profile_technologies WHERE profile_id = ?1", [&initialization.profile_id]).map_err(|error| format!("Não foi possível atualizar as tecnologias: {error}"))?;
    for (index, (slug, label)) in technologies.iter().enumerate() { transaction.execute("INSERT INTO profile_technologies(profile_id, slug, label, sort_order, created_at) VALUES(?1, ?2, ?3, ?4, ?5)", params![initialization.profile_id, slug, label, index as i64, now]).map_err(|error| format!("Não foi possível salvar uma tecnologia: {error}"))?; }
    transaction.commit().map_err(|error| format!("Não foi possível confirmar o Profile: {error}"))?;
    get_local_profile(app)
}

fn profile_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { Some("image/png") } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) { Some("image/jpeg") } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") { Some("image/webp") } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") { (!bytes.windows(11).any(|window| window == b"NETSCAPE2.0")).then_some("image/gif") } else { None }
}

#[tauri::command]
fn replace_profile_asset(app: AppHandle, input: ReplaceProfileAsset) -> Result<LocalProfile, String> {
    if input.kind != "avatar" && input.kind != "cover" { return Err("Escolha avatar ou cover.".to_string()); }
    if input.original_name.trim().is_empty() { return Err("A imagem precisa ter um nome de arquivo.".to_string()); }
    let maximum = if input.kind == "avatar" { 10_000_000 } else { 15_000_000 };
    if input.bytes.is_empty() || input.bytes.len() > maximum { return Err("A imagem está vazia ou excede o limite permitido.".to_string()); }
    let media_type = profile_media_type(&input.bytes).ok_or_else(|| "Use PNG, JPEG, WebP ou GIF não animado.".to_string())?;
    let initialization = initialize_local_profile(app.clone())?; let id = Uuid::now_v7().to_string(); let relative = format!("profiles/{}/assets/{}", initialization.profile_id, id); let path = data_dir(&app)?.join(&relative);
    let parent = path.parent().ok_or_else(|| "Não foi possível preparar o diretório da imagem.".to_string())?; fs::create_dir_all(parent).map_err(|error| format!("Não foi possível criar o diretório da imagem: {error}"))?; fs::write(&path, &input.bytes).map_err(|error| format!("Não foi possível copiar a imagem: {error}"))?;
    let catalog = open_catalog(&app)?; ensure_profile_details(&catalog, &initialization.profile_id)?; let now = timestamp()?;
    let changed = (|| -> Result<(), String> { let transaction = catalog.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a atualização da imagem: {error}"))?; transaction.execute("INSERT INTO profile_assets(id, profile_id, kind, stored_path, media_type, byte_size, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![id, initialization.profile_id, input.kind, relative, media_type, input.bytes.len() as i64, now]).map_err(|error| format!("Não foi possível registrar a imagem: {error}"))?; let column = if input.kind == "avatar" { "avatar_asset_id" } else { "cover_asset_id" }; let old: Option<String> = transaction.query_row(&format!("SELECT {column} FROM profile_details WHERE profile_id = ?1"), [&initialization.profile_id], |row| row.get(0)).optional().map_err(|error| format!("Não foi possível localizar a imagem anterior: {error}"))?.flatten(); transaction.execute(&format!("UPDATE profile_details SET {column} = ?1, updated_at = ?2 WHERE profile_id = ?3"), params![id, now, initialization.profile_id]).map_err(|error| format!("Não foi possível atualizar a imagem do Profile: {error}"))?; if let Some(old) = old { transaction.execute("UPDATE profile_assets SET deleted_at = ?1 WHERE id = ?2", params![now, old]).map_err(|error| format!("Não foi possível preservar a imagem anterior: {error}"))?; } transaction.commit().map_err(|error| format!("Não foi possível confirmar a imagem: {error}"))?; Ok(()) })();
    if let Err(error) = changed { let _ = fs::remove_file(path); return Err(error); }
    get_local_profile(app)
}

#[tauri::command]
fn read_profile_asset(app: AppHandle, asset_id: String) -> Result<ProfileAssetBinary, String> {
    let initialization = initialize_local_profile(app.clone())?; let catalog = open_catalog(&app)?;
    let (stored_path, media_type): (String, String) = catalog.query_row("SELECT stored_path, media_type FROM profile_assets WHERE id = ?1 AND profile_id = ?2 AND deleted_at IS NULL", params![asset_id, initialization.profile_id], |row| Ok((row.get(0)?, row.get(1)?))).map_err(|error| format!("Não foi possível localizar a imagem do Profile: {error}"))?;
    Ok(ProfileAssetBinary { media_type, bytes: fs::read(data_dir(&app)?.join(stored_path)).map_err(|error| format!("Não foi possível ler a imagem do Profile: {error}"))? })
}

#[tauri::command]
fn remove_profile_asset(app: AppHandle, kind: String) -> Result<LocalProfile, String> {
    if kind != "avatar" && kind != "cover" { return Err("Escolha avatar ou cover.".to_string()); }
    let initialization = initialize_local_profile(app.clone())?; let catalog = open_catalog(&app)?; ensure_profile_details(&catalog, &initialization.profile_id)?; let column = if kind == "avatar" { "avatar_asset_id" } else { "cover_asset_id" }; let now = timestamp()?;
    let transaction = catalog.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a remoção da imagem: {error}"))?;
    let asset_id: Option<String> = transaction.query_row(&format!("SELECT {column} FROM profile_details WHERE profile_id = ?1"), [&initialization.profile_id], |row| row.get(0)).optional().map_err(|error| format!("Não foi possível localizar a imagem: {error}"))?.flatten();
    transaction.execute(&format!("UPDATE profile_details SET {column} = NULL, updated_at = ?1 WHERE profile_id = ?2"), params![now, initialization.profile_id]).map_err(|error| format!("Não foi possível remover a imagem do Profile: {error}"))?;
    if let Some(asset_id) = asset_id { transaction.execute("UPDATE profile_assets SET deleted_at = ?1 WHERE id = ?2", params![now, asset_id]).map_err(|error| format!("Não foi possível preservar a imagem removida: {error}"))?; }
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a remoção da imagem: {error}"))?;
    get_local_profile(app)
}

fn active_profile_projects(catalog: &Connection, profile_id: &str) -> Result<Vec<LocalProjectSummary>, String> {
    let mut statement = catalog.prepare("SELECT p.id, p.workspace_id, p.name, p.storage_mode, p.created_at, p.updated_at FROM projects p JOIN workspaces w ON w.id = p.workspace_id WHERE w.profile_id = ?1 AND w.deleted_at IS NULL AND p.deleted_at IS NULL ORDER BY p.updated_at DESC")
        .map_err(|error| format!("Não foi possível preparar os Projects do Profile: {error}"))?;
    let projects = statement.query_map([profile_id], |row| Ok(LocalProjectSummary { id: row.get(0)?, workspace_id: row.get(1)?, name: row.get(2)?, storage_mode: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? }))
        .map_err(|error| format!("Não foi possível listar os Projects do Profile: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler os Projects do Profile: {error}"))?;
    Ok(projects)
}

fn profile_resources(app: &AppHandle, profile_id: &str, tab: &str) -> Result<(Vec<ProfileResource>, i64), String> {
    let catalog = open_catalog(app)?; let projects = active_profile_projects(&catalog, profile_id)?; let mut items = Vec::new(); let mut unavailable = 0;
    if tab == "projects" { items = projects.iter().map(|project| ProfileResource { id: project.id.clone(), project_id: Some(project.id.clone()), title: project.name.clone(), subtitle: if project.storage_mode == "local" { "Project local".to_string() } else { "Project cloud".to_string() }, kind: "project".to_string(), occurred_at: project.updated_at }).collect(); return Ok((items, 0)); }
    if tab == "templates" { let mut statement = catalog.prepare("SELECT template_id, created_at FROM template_favorites WHERE profile_id = ?1 ORDER BY created_at DESC").map_err(|error| format!("Não foi possível preparar os Templates favoritos: {error}"))?; items = statement.query_map([profile_id], |row| Ok(ProfileResource { id: row.get(0)?, project_id: None, title: "Template favorito".to_string(), subtitle: "Abra Templates para usar ou remover o favorito".to_string(), kind: "template".to_string(), occurred_at: row.get(1)? })).map_err(|error| format!("Não foi possível listar os Templates favoritos: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler os Templates favoritos: {error}"))?; return Ok((items, 0)); }
    if tab == "components" { let mut statement = catalog.prepare("SELECT source_key, added_at FROM library_components WHERE profile_id = ?1 AND removed_at IS NULL ORDER BY added_at DESC").map_err(|error| format!("Não foi possível preparar os Components: {error}"))?; items = statement.query_map([profile_id], |row| Ok(ProfileResource { id: row.get(0)?, project_id: None, title: row.get(0)?, subtitle: "Biblioteca pessoal".to_string(), kind: "component".to_string(), occurred_at: row.get(1)? })).map_err(|error| format!("Não foi possível listar os Components: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler os Components: {error}"))?; return Ok((items, 0)); }
    for project in projects {
        let path: Option<String> = catalog.query_row("SELECT relative_path FROM projects WHERE id = ?1", [&project.id], |row| row.get(0)).optional().map_err(|error| format!("Não foi possível localizar um Project: {error}"))?;
        let Some(path) = path else { unavailable += 1; continue; };
        let Ok(database) = Connection::open(data_dir(app)?.join(path).join(PROJECT_DATABASE)) else { unavailable += 1; continue; };
        let query = match tab { "notes" => "SELECT id, title, updated_at FROM notes WHERE deleted_at IS NULL AND is_archived = 0", "docs" => "SELECT id, title, updated_at FROM docs WHERE deleted_at IS NULL AND kind = 'page'", "simulations" => "SELECT id, scenario, created_at FROM simulation_runs", "activity" => "SELECT id, entity_type || ':' || operation, created_at FROM revisions", _ => return Err("A aba de Profile não existe.".to_string()) };
        let kind = if tab == "activity" { "activity" } else { tab };
        let mut statement = match database.prepare(query) { Ok(value) => value, Err(_) => { unavailable += 1; continue; } };
        let mapped = statement.query_map([], |row| Ok(ProfileResource { id: row.get(0)?, project_id: Some(project.id.clone()), title: row.get(1)?, subtitle: project.name.clone(), kind: kind.to_string(), occurred_at: row.get(2)? }));
        match mapped { Ok(rows) => items.extend(rows.filter_map(Result::ok)), Err(_) => unavailable += 1 }
    }
    items.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at).then_with(|| right.id.cmp(&left.id)));
    Ok((items, unavailable))
}

#[tauri::command]
fn get_profile_overview(app: AppHandle) -> Result<ProfileOverview, String> {
    let initialization = initialize_local_profile(app.clone())?; let catalog = open_catalog(&app)?; let profile = read_local_profile(&catalog, &initialization.profile_id)?; let projects = active_profile_projects(&catalog, &initialization.profile_id)?;
    let (notes, unavailable_notes) = profile_resources(&app, &initialization.profile_id, "notes")?; let (simulations, unavailable_simulations) = profile_resources(&app, &initialization.profile_id, "simulations")?; let (activity, unavailable_activity) = profile_resources(&app, &initialization.profile_id, "activity")?;
    let favorites: i64 = catalog.query_row("SELECT COUNT(*) FROM template_favorites WHERE profile_id = ?1", [&initialization.profile_id], |row| row.get(0)).map_err(|error| format!("Não foi possível contar os Templates favoritos: {error}"))?;
    Ok(ProfileOverview { profile, stats: vec![ProfileStat { label: "Projects".to_string(), value: projects.len() as i64, detail: "Ideias em construção".to_string() }, ProfileStat { label: "Notes".to_string(), value: notes.len() as i64, detail: "Conhecimento registrado".to_string() }, ProfileStat { label: "Templates".to_string(), value: favorites, detail: "Favoritos pessoais".to_string() }, ProfileStat { label: "Simulações".to_string(), value: simulations.len() as i64, detail: "Execuções salvas".to_string() }], recent_projects: projects.into_iter().take(5).collect(), activity: activity.into_iter().take(5).collect(), unavailable_project_count: unavailable_notes.max(unavailable_simulations).max(unavailable_activity) })
}

#[tauri::command]
fn list_profile_resources(app: AppHandle, input: ProfileResourceQuery) -> Result<ProfileResourcePage, String> {
    let initialization = initialize_local_profile(app.clone())?; let limit = input.limit.unwrap_or(20).clamp(1, 20); let (items, unavailable) = profile_resources(&app, &initialization.profile_id, &input.tab)?;
    let start = input.cursor.as_deref().and_then(|cursor| cursor.parse::<usize>().ok()).unwrap_or(0); let end = (start + limit).min(items.len());
    Ok(ProfileResourcePage { items: items[start..end].to_vec(), next_cursor: (end < items.len()).then_some(end.to_string()), unavailable_project_count: unavailable })
}

fn is_official_template(template_id: &str) -> bool {
    matches!(template_id,
        "web-application-architecture" | "microservices-architecture" | "aws-infrastructure" |
        "nextjs-application" | "api-architecture" | "database-schema" |
        "daily-planner" | "study-notes" | "meeting-notes"
    )
}

#[tauri::command]
fn list_templates(app: AppHandle) -> Result<TemplateCatalogState, String> {
    let initialization = initialize_local_profile(app.clone())?;
    let connection = open_catalog(&app)?;
    let mut statement = connection.prepare("SELECT template_id FROM template_favorites WHERE profile_id = ?1 ORDER BY created_at DESC")
        .map_err(|error| format!("Não foi possível preparar os favoritos de Templates: {error}"))?;
    let favorite_template_ids = statement.query_map([initialization.profile_id], |row| row.get(0))
        .map_err(|error| format!("Não foi possível listar os favoritos de Templates: {error}"))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|error| format!("Não foi possível ler os favoritos de Templates: {error}"))?;
    Ok(TemplateCatalogState { favorite_template_ids })
}

#[tauri::command]
fn set_template_favorite(app: AppHandle, template_id: String, favorite: bool) -> Result<TemplateCatalogState, String> {
    let template_id = template_id.trim();
    if !is_official_template(template_id) { return Err("O Template solicitado não existe no catálogo oficial.".to_string()); }
    let initialization = initialize_local_profile(app.clone())?;
    let connection = open_catalog(&app)?;
    if favorite {
        connection.execute("INSERT OR IGNORE INTO template_favorites(profile_id, template_id, created_at) VALUES(?1, ?2, ?3)", params![initialization.profile_id, template_id, timestamp()?])
            .map_err(|error| format!("Não foi possível favoritar o Template: {error}"))?;
    } else {
        connection.execute("DELETE FROM template_favorites WHERE profile_id = ?1 AND template_id = ?2", params![initialization.profile_id, template_id])
            .map_err(|error| format!("Não foi possível remover o favorito do Template: {error}"))?;
    }
    list_templates(app)
}

fn persist_template_content(connection: &mut Connection, project_id: &str, template_id: &str, seed: &TemplateSeed, now: i64) -> Result<String, String> {
    if seed.version.trim().is_empty() || seed.canvas_name.trim().is_empty() { return Err("O manifesto do Template está incompleto.".to_string()); }
    let canvas_id = Uuid::now_v7().to_string();
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar o conteúdo do Template: {error}"))?;
    transaction.execute("INSERT INTO canvases(id, project_id, name, viewport, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?5)", params![canvas_id, project_id, seed.canvas_name.trim(), json!({"x": 0.0, "y": 0.0, "zoom": 1.0}).to_string(), now])
        .map_err(|error| format!("Não foi possível criar o Canvas do Template: {error}"))?;
    let mut component_ids = Vec::new();
    let mut ports = Vec::new();
    for component in &seed.components {
        if component.label.trim().is_empty() || component.component_type.trim().is_empty() { return Err("Todo Component do Template precisa de tipo e nome.".to_string()); }
        let component_id = Uuid::now_v7().to_string();
        transaction.execute("INSERT INTO canvas_elements(id, canvas_id, kind, type, x, y, width, height, data, created_at, updated_at) VALUES(?1, ?2, 'component', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)", params![component_id, canvas_id, component.component_type.trim(), component.x, component.y, component.width.max(80.0), component.height.max(56.0), component.data.to_string(), now])
            .map_err(|error| format!("Não foi possível criar um Component do Template: {error}"))?;
        transaction.execute("INSERT INTO components(element_id, label, description, color, icon, properties) VALUES(?1, ?2, ?3, ?4, ?5, ?6)", params![component_id, component.label.trim(), component.description, component.color, component.component_type.trim(), component.data.to_string()])
            .map_err(|error| format!("Não foi possível salvar um Component do Template: {error}"))?;
        let mut component_ports = Vec::new();
        for (index, port) in component.ports.iter().enumerate() {
            if port.key.trim().is_empty() { return Err("Todo Port do Template precisa de nome.".to_string()); }
            let port_id = Uuid::now_v7().to_string();
            let port_data = if port.data.is_null() { json!({}) } else { port.data.clone() };
            transaction.execute("INSERT INTO component_ports(id, component_id, key, direction, protocol, data, order_index) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![port_id, component_id, port.key.trim(), port.direction, port.protocol, port_data.to_string(), index as i64])
                .map_err(|error| format!("Não foi possível salvar um Port do Template: {error}"))?;
            component_ports.push((port.key.clone(), port_id));
        }
        component_ids.push(component_id);
        ports.push(component_ports);
    }
    for connection_seed in &seed.connections {
        let source_component_id = component_ids.get(connection_seed.source_index).ok_or_else(|| "Uma conexão do Template aponta para um Component inexistente.".to_string())?;
        let target_component_id = component_ids.get(connection_seed.target_index).ok_or_else(|| "Uma conexão do Template aponta para um Component inexistente.".to_string())?;
        let source_port_id = ports.get(connection_seed.source_index).and_then(|items| items.iter().find(|(key, _)| key == &connection_seed.source_port_key)).map(|(_, id)| id).ok_or_else(|| "Uma conexão do Template aponta para um Port de origem inexistente.".to_string())?;
        let target_port_id = ports.get(connection_seed.target_index).and_then(|items| items.iter().find(|(key, _)| key == &connection_seed.target_port_key)).map(|(_, id)| id).ok_or_else(|| "Uma conexão do Template aponta para um Port de destino inexistente.".to_string())?;
        transaction.execute("INSERT INTO connections(id, canvas_id, source_component_id, source_port_id, target_component_id, target_port_id, type, data, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '{}', ?8, ?8)", params![Uuid::now_v7().to_string(), canvas_id, source_component_id, source_port_id, target_component_id, target_port_id, connection_seed.connection_type, now])
            .map_err(|error| format!("Não foi possível criar uma conexão do Template: {error}"))?;
    }
    for note in &seed.notes {
        transaction.execute("INSERT INTO notes(id, project_id, title, content, content_format, tags, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, 'markdown', ?5, ?6, ?6)", params![Uuid::now_v7().to_string(), project_id, note.title, note.content, serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string()), now])
            .map_err(|error| format!("Não foi possível criar uma Note do Template: {error}"))?;
    }
    for doc in &seed.docs {
        transaction.execute("INSERT INTO docs(id, project_id, title, content, content_format, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, 'markdown', ?5, ?5)", params![Uuid::now_v7().to_string(), project_id, doc.title, doc.content, now])
            .map_err(|error| format!("Não foi possível criar um Doc do Template: {error}"))?;
    }
    append_revision(&transaction, project_id, "project", "created_from_template", json!({"templateId": template_id, "templateVersion": seed.version}), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar o conteúdo do Template: {error}"))?;
    Ok(canvas_id)
}

#[tauri::command]
fn create_project_from_template(app: AppHandle, input: CreateProjectFromTemplate) -> Result<TemplateProjectResult, String> {
    let project_name = input.project_name.trim();
    let template_id = input.template_id.trim();
    if project_name.is_empty() { return Err("O nome do Project é obrigatório.".to_string()); }
    if !is_official_template(template_id) { return Err("O Template solicitado não existe no catálogo oficial.".to_string()); }

    let initialization = initialize_local_profile(app.clone())?;
    let project_id = Uuid::now_v7().to_string();
    let now = timestamp()?;
    let project_directory = data_dir(&app)?.join("projects").join(&project_id);
    let database_path = project_directory.join(PROJECT_DATABASE);
    let created = (|| -> Result<String, String> {
        fs::create_dir_all(project_directory.join("assets")).map_err(|error| format!("Não foi possível criar os assets do Project: {error}"))?;
        fs::create_dir_all(project_directory.join("backups")).map_err(|error| format!("Não foi possível criar os backups do Project: {error}"))?;
        migrate_project_database(&database_path)?;
        let mut project = Connection::open(&database_path).map_err(|error| format!("Não foi possível abrir o banco do Project: {error}"))?;
        project.execute_batch("PRAGMA foreign_keys = ON;").map_err(|error| format!("Não foi possível preparar o banco do Project: {error}"))?;
        let canvas_id = persist_template_content(&mut project, &project_id, template_id, &input.seed, now)?;

        let catalog = open_catalog(&app)?;
        catalog.execute("INSERT INTO projects(id, workspace_id, name, storage_mode, relative_path, created_at, updated_at) VALUES(?1, ?2, ?3, 'local', ?4, ?5, ?5)", params![project_id, initialization.workspace_id, project_name, format!("projects/{project_id}"), now])
            .map_err(|error| format!("Não foi possível registrar o Project no catálogo: {error}"))?;
        if template_id == "aws-infrastructure" { enable_aws_library(&catalog, &initialization.profile_id)?; }
        Ok(canvas_id)
    })();

    match created {
        Ok(canvas_id) => Ok(TemplateProjectResult { project_id, canvas_id }),
        Err(error) => {
            if let Ok(catalog) = open_catalog(&app) { let _ = catalog.execute("DELETE FROM projects WHERE id = ?1", [&project_id]); }
            let _ = fs::remove_dir_all(&project_directory);
            Err(error)
        }
    }
}

#[tauri::command]
fn apply_template_to_project(app: AppHandle, input: ApplyTemplateToProject) -> Result<TemplateProjectResult, String> {
    let template_id = input.template_id.trim();
    let project_id = input.project_id.trim();
    if project_id.is_empty() { return Err("Escolha o Project que receberá o Template.".to_string()); }
    if !is_official_template(template_id) { return Err("O Template solicitado não existe no catálogo oficial.".to_string()); }
    let initialization = initialize_local_profile(app.clone())?;
    let now = timestamp()?;
    if template_id == "aws-infrastructure" {
        let catalog = open_catalog(&app)?;
        enable_aws_library(&catalog, &initialization.profile_id)?;
    }
    let mut project = open_project_database(&app, project_id)?;
    let canvas_id = persist_template_content(&mut project, project_id, template_id, &input.seed, now)?;
    Ok(TemplateProjectResult { project_id: project_id.to_string(), canvas_id })
}

#[tauri::command]
fn list_personal_library(app: AppHandle) -> Result<Vec<PersonalLibraryItem>, String> {
    let connection = open_catalog(&app)?;
    let profile_id = local_profile_id(&connection)?;
    ensure_component_library(&connection, &profile_id)?;
    let mut statement = connection.prepare(
        "SELECT item.id, item.source_key, EXISTS(SELECT 1 FROM component_favorites favorite WHERE favorite.library_component_id = item.id), item.added_at
         FROM library_components item
         WHERE item.profile_id = ?1 AND item.removed_at IS NULL
         ORDER BY item.added_at",
    ).map_err(|error| format!("Não foi possível preparar a biblioteca pessoal: {error}"))?;
    let items = statement.query_map([profile_id], |row| Ok(PersonalLibraryItem {
        id: row.get(0)?, source_key: row.get(1)?, is_favorite: row.get(2)?, added_at: row.get(3)?,
    })).map_err(|error| format!("Não foi possível listar a biblioteca pessoal: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Não foi possível ler a biblioteca pessoal: {error}"))?;
    Ok(items)
}

#[tauri::command]
fn add_library_component(app: AppHandle, source_key: String) -> Result<PersonalLibraryItem, String> {
    let source_key = source_key.trim();
    if source_key.is_empty() { return Err("O Component é obrigatório.".to_string()); }
    let connection = open_catalog(&app)?;
    let profile_id = local_profile_id(&connection)?;
    ensure_component_library(&connection, &profile_id)?;
    let now = timestamp()?;
    let existing = connection.query_row(
        "SELECT id, source_key, EXISTS(SELECT 1 FROM component_favorites favorite WHERE favorite.library_component_id = library_components.id), added_at
         FROM library_components WHERE profile_id = ?1 AND source_key = ?2",
        params![profile_id, source_key],
        |row| Ok(PersonalLibraryItem { id: row.get(0)?, source_key: row.get(1)?, is_favorite: row.get(2)?, added_at: row.get(3)? }),
    ).optional().map_err(|error| format!("Não foi possível consultar o Component: {error}"))?;
    if let Some(item) = existing { return Ok(item); }
    let id = Uuid::now_v7().to_string();
    connection.execute("INSERT INTO library_components(id, profile_id, source_key, added_at) VALUES(?1, ?2, ?3, ?4)", params![id, profile_id, source_key, now])
        .map_err(|error| format!("Não foi possível adicionar o Component à biblioteca: {error}"))?;
    Ok(PersonalLibraryItem { id, source_key: source_key.to_string(), is_favorite: false, added_at: now })
}

#[tauri::command]
fn create_local_project(app: AppHandle, name: String) -> Result<LocalProjectSummary, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("O nome do Project é obrigatório.".to_string());
    }
    let initialization = initialize_local_profile(app.clone())?;
    let id = Uuid::now_v7().to_string();
    let now = timestamp()?;
    let project_dir = data_dir(&app)?.join("projects").join(&id);
    let database_path = project_dir.join(PROJECT_DATABASE);
    fs::create_dir_all(project_dir.join("assets"))
        .map_err(|error| format!("Não foi possível criar os assets do Project: {error}"))?;
    fs::create_dir_all(project_dir.join("backups"))
        .map_err(|error| format!("Não foi possível criar os backups do Project: {error}"))?;
    let result = (|| -> Result<(), String> {
        migrate_project_database(&database_path)?;
        let project = Connection::open(&database_path)
            .map_err(|error| format!("Não foi possível abrir o banco do Project: {error}"))?;
        let transaction = project
            .unchecked_transaction()
            .map_err(|error| format!("Não foi possível iniciar o Project: {error}"))?;
        transaction.execute("INSERT INTO project_metadata(project_id, created_at, updated_at) VALUES(?1, ?2, ?2)", params![id, now]).map_err(|error| format!("Não foi possível registrar a metadata do Project: {error}"))?;
        transaction.execute("INSERT INTO revisions(id, entity_id, entity_type, operation, payload, created_at) VALUES(?1, ?2, 'project', 'created', ?3, ?4)", params![Uuid::now_v7().to_string(), id, serde_json::json!({"name": name, "storageMode": "local"}).to_string(), now]).map_err(|error| format!("Não foi possível registrar a revision inicial: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Não foi possível confirmar o banco do Project: {error}"))?;
        let catalog = open_catalog(&app)?;
        catalog.execute("INSERT INTO projects(id, workspace_id, name, storage_mode, relative_path, created_at, updated_at) VALUES(?1, ?2, ?3, 'local', ?4, ?5, ?5)", params![id, initialization.workspace_id, name, format!("projects/{id}"), now]).map_err(|error| format!("Não foi possível registrar o Project no catálogo: {error}"))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&project_dir);
        return Err(error);
    }
    Ok(LocalProjectSummary {
        id,
        workspace_id: initialization.workspace_id,
        name: name.to_string(),
        storage_mode: "local".to_string(),
        created_at: now,
        updated_at: now,
    })
}

fn local_project_directory(root: &Path, relative_path: &str, project_id: &str) -> Result<PathBuf, String> {
    let expected = Path::new("projects").join(project_id);
    if Path::new(relative_path) != expected.as_path() {
        return Err("O caminho do Project local não é seguro para mover à lixeira.".to_string());
    }
    Ok(root.join(expected))
}

#[tauri::command]
fn trash_local_project(app: AppHandle, project_id: String) -> Result<(), String> {
    let project_id = project_id.trim();
    if project_id.is_empty() { return Err("O Project é obrigatório.".to_string()); }
    let root = data_dir(&app)?;
    let connection = open_catalog(&app)?;
    let relative_path: String = connection.query_row(
        "SELECT relative_path FROM projects WHERE id = ?1 AND deleted_at IS NULL",
        [project_id],
        |row| row.get(0),
    ).optional().map_err(|error| format!("Não foi possível localizar o Project: {error}"))?
        .ok_or_else(|| "O Project solicitado não existe ou já está na lixeira.".to_string())?;
    let source = local_project_directory(&root, &relative_path, project_id)?;
    if !source.is_dir() { return Err("Os arquivos locais deste Project não foram encontrados.".to_string()); }
    let target = root.join("trash").join("projects").join(project_id);
    if target.exists() { return Err("Já existe uma cópia deste Project na lixeira local.".to_string()); }
    let target_parent = target.parent().ok_or_else(|| "Não foi possível preparar a lixeira local.".to_string())?;
    fs::create_dir_all(target_parent).map_err(|error| format!("Não foi possível preparar a lixeira local: {error}"))?;
    fs::rename(&source, &target).map_err(|error| format!("Não foi possível mover o Project para a lixeira: {error}"))?;
    let now = match timestamp() {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::rename(&target, &source);
            return Err(error);
        }
    };
    match connection.execute(
        "UPDATE projects SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, project_id],
    ) {
        Ok(1) => Ok(()),
        Ok(_) | Err(_) => {
            let _ = fs::rename(&target, &source);
            Err("Não foi possível confirmar a exclusão do Project. Nenhum arquivo foi removido.".to_string())
        }
    }
}

#[tauri::command]
fn list_local_projects(app: AppHandle) -> Result<Vec<LocalProjectSummary>, String> {
    let connection = open_catalog(&app)?;
    let mut statement = connection.prepare("SELECT id, workspace_id, name, storage_mode, created_at, updated_at FROM projects WHERE deleted_at IS NULL ORDER BY updated_at DESC").map_err(|error| format!("Não foi possível listar os Projects locais: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(LocalProjectSummary {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                name: row.get(2)?,
                storage_mode: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|error| format!("Não foi possível consultar os Projects locais: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Não foi possível ler os Projects locais: {error}"))
}

#[tauri::command]
fn load_canvas(app: AppHandle, project_id: String, canvas_id: Option<String>) -> Result<CanvasSnapshot, String> {
    let mut connection = open_project_database(&app, &project_id)?;
    load_canvas_snapshot(&mut connection, &project_id, canvas_id.as_deref())
}

#[tauri::command]
fn create_canvas_component(
    app: AppHandle,
    input: CreateCanvasComponent,
) -> Result<CanvasSnapshot, String> {
    if input.label.trim().is_empty() || input.ports.iter().any(|port| port.key.trim().is_empty()) {
        return Err("Todo Component e Port precisa de um nome.".to_string());
    }
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let id = Uuid::now_v7().to_string();
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a criação do Component: {error}"))?;
    transaction.execute(
        "INSERT INTO canvas_elements(id, canvas_id, kind, type, x, y, width, height, data, created_at, updated_at) VALUES(?1, ?2, 'component', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![id, input.canvas_id, input.component_type, input.x, input.y, input.width.max(80.0), input.height.max(56.0), input.data.to_string(), now],
    ).map_err(|error| format!("Não foi possível criar o elemento do Canvas: {error}"))?;
    transaction.execute(
        "INSERT INTO components(element_id, label, description, color, icon, properties) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, input.label, input.description, input.color, input.component_type, input.data.to_string()],
    ).map_err(|error| format!("Não foi possível criar o Component: {error}"))?;
    for (index, port) in input.ports.iter().enumerate() {
        transaction.execute(
            "INSERT INTO component_ports(id, component_id, key, direction, protocol, data, order_index) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![Uuid::now_v7().to_string(), id, port.key, port.direction, port.protocol, port.data.to_string(), index as i64],
        ).map_err(|error| format!("Não foi possível criar o Port do Component: {error}"))?;
    }
    append_revision(
        &transaction,
        &id,
        "component",
        "created",
        json!({ "canvasId": input.canvas_id, "type": input.component_type }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar o Component: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id, Some(&input.canvas_id))
}

#[tauri::command]
fn create_canvas_visual(app: AppHandle, input: CreateCanvasVisual) -> Result<CanvasSnapshot, String> {
    if input.visual_type != "image" { return Err("Somente imagens visuais são suportadas nesta etapa.".to_string()); }
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?; let id = Uuid::now_v7().to_string();
    connection.execute("INSERT INTO canvas_elements(id, canvas_id, kind, type, x, y, width, height, data, created_at, updated_at) VALUES(?1, ?2, 'visual', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)", params![id, input.canvas_id, input.visual_type, input.x, input.y, input.width.max(80.0), input.height.max(60.0), input.data.to_string(), now]).map_err(|error| format!("Não foi possível criar a imagem no Canvas: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id, Some(&input.canvas_id))
}

#[tauri::command]
fn update_canvas_visual(app: AppHandle, input: UpdateCanvasVisual) -> Result<(), String> {
    let connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a edição da imagem: {error}"))?;
    let changed = transaction.execute(
        "UPDATE canvas_elements SET data = ?1, updated_at = ?2 WHERE id = ?3 AND canvas_id = ?4 AND kind = 'visual' AND deleted_at IS NULL",
        params![input.data.to_string(), now, input.visual_id, input.canvas_id],
    ).map_err(|error| format!("Não foi possível editar a imagem no Canvas: {error}"))?;
    if changed != 1 { return Err("A imagem solicitada não existe no Canvas.".to_string()); }
    append_revision(&transaction, &input.visual_id, "visual", "updated", json!({ "canvasId": input.canvas_id }), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a edição da imagem: {error}"))?;
    Ok(())
}

#[tauri::command]
fn delete_canvas_visual(app: AppHandle, input: DeleteCanvasVisual) -> Result<CanvasSnapshot, String> {
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a remoção da imagem: {error}"))?;
    let changed = transaction.execute(
        "UPDATE canvas_elements SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND canvas_id = ?3 AND kind = 'visual' AND deleted_at IS NULL",
        params![now, input.visual_id, input.canvas_id],
    ).map_err(|error| format!("Não foi possível remover a imagem do Canvas: {error}"))?;
    if changed != 1 { return Err("A imagem solicitada não existe no Canvas.".to_string()); }
    append_revision(&transaction, &input.visual_id, "visual", "deleted", json!({ "canvasId": input.canvas_id }), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a remoção da imagem: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id, Some(&input.canvas_id))
}

#[tauri::command]
fn move_canvas_element(app: AppHandle, input: MoveCanvasElement) -> Result<(), String> {
    let connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar o movimento do Component: {error}"))?;
    let changed = transaction.execute("UPDATE canvas_elements SET x = ?1, y = ?2, updated_at = ?3 WHERE id = ?4 AND canvas_id = ?5 AND deleted_at IS NULL", params![input.x, input.y, now, input.element_id, input.canvas_id]).map_err(|error| format!("Não foi possível mover o Component: {error}"))?;
    if changed != 1 {
        return Err("O Component solicitado não existe no Canvas.".to_string());
    }
    append_revision(
        &transaction,
        &input.element_id,
        "component",
        "moved",
        json!({ "x": input.x, "y": input.y }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar o movimento do Component: {error}"))
}

#[tauri::command]
fn update_canvas_component(app: AppHandle, input: UpdateCanvasComponent) -> Result<(), String> {
    let connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a edição do Component: {error}"))?;
    let changed = transaction.execute("UPDATE components SET label = ?1, description = ?2, color = ?3, properties = ?4 WHERE element_id = ?5 AND EXISTS(SELECT 1 FROM canvas_elements WHERE id = ?5 AND canvas_id = ?6 AND deleted_at IS NULL)", params![input.label, input.description, input.color, input.data.to_string(), input.component_id, input.canvas_id]).map_err(|error| format!("Não foi possível editar o Component: {error}"))?;
    if changed != 1 {
        return Err("O Component solicitado não existe no Canvas.".to_string());
    }
    transaction
        .execute(
            "UPDATE canvas_elements SET data = ?1, updated_at = ?2 WHERE id = ?3",
            params![input.data.to_string(), now, input.component_id],
        )
        .map_err(|error| format!("Não foi possível atualizar os dados do Component: {error}"))?;
    append_revision(
        &transaction,
        &input.component_id,
        "component",
        "updated",
        json!({ "label": input.label }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar a edição do Component: {error}"))
}

#[tauri::command]
fn delete_canvas_component(
    app: AppHandle,
    input: DeleteCanvasComponent,
) -> Result<CanvasSnapshot, String> {
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a remoção do Component: {error}"))?;
    let changed = transaction
        .execute(
            "UPDATE canvas_elements SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND canvas_id = ?3 AND kind = 'component' AND deleted_at IS NULL",
            params![now, input.component_id, input.canvas_id],
        )
        .map_err(|error| format!("Não foi possível remover o Component: {error}"))?;
    if changed != 1 {
        return Err("O Component solicitado não existe no Canvas.".to_string());
    }
    let removed_connections = transaction
        .execute(
            "UPDATE connections SET deleted_at = ?1, updated_at = ?1 WHERE canvas_id = ?2 AND deleted_at IS NULL AND (source_component_id = ?3 OR target_component_id = ?3)",
            params![now, input.canvas_id, input.component_id],
        )
        .map_err(|error| format!("Não foi possível remover as Connections do Component: {error}"))?;
    append_revision(
        &transaction,
        &input.component_id,
        "component",
        "deleted",
        json!({ "canvasId": input.canvas_id, "removedConnections": removed_connections }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar a remoção do Component: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id, Some(&input.canvas_id))
}

#[tauri::command]
fn create_canvas_connection(
    app: AppHandle,
    input: CreateCanvasConnection,
) -> Result<CanvasSnapshot, String> {
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let endpoint_count: i64 = connection.query_row("SELECT COUNT(*) FROM component_ports p JOIN canvas_elements e ON e.id = p.component_id WHERE e.canvas_id = ?1 AND e.deleted_at IS NULL AND ((p.id = ?2 AND p.component_id = ?3) OR (p.id = ?4 AND p.component_id = ?5))", params![input.canvas_id, input.source_port_id, input.source_component_id, input.target_port_id, input.target_component_id], |row| row.get(0)).map_err(|error| format!("Não foi possível validar os Ports: {error}"))?;
    if endpoint_count != 2 || input.source_component_id == input.target_component_id {
        return Err(
            "A Connection precisa usar Ports de Components diferentes no mesmo Canvas.".to_string(),
        );
    }
    let now = timestamp()?;
    let id = Uuid::now_v7().to_string();
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("Não foi possível iniciar a Connection: {error}"))?;
    transaction.execute("INSERT INTO connections(id, canvas_id, source_component_id, source_port_id, target_component_id, target_port_id, type, data, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)", params![id, input.canvas_id, input.source_component_id, input.source_port_id, input.target_component_id, input.target_port_id, input.connection_type, input.data.to_string(), now]).map_err(|error| format!("Não foi possível criar a Connection: {error}"))?;
    append_revision(
        &transaction,
        &id,
        "connection",
        "created",
        json!({ "canvasId": input.canvas_id }),
        now,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("Não foi possível confirmar a Connection: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id, Some(&input.canvas_id))
}

fn read_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteSummary> {
    let tags: String = row.get(7)?;
    Ok(NoteSummary { id: row.get(0)?, project_id: row.get(1)?, title: row.get(2)?, content: row.get(3)?, content_format: row.get(4)?, is_pinned: row.get(5)?, is_archived: row.get(6)?, tags: serde_json::from_str(&tags).unwrap_or_default(), created_at: row.get(8)?, updated_at: row.get(9)? })
}

#[tauri::command]
fn create_asset(app: AppHandle, input: CreateAsset) -> Result<AssetSummary, String> {
    let allowed = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/svg+xml"];
    if !allowed.contains(&input.media_type.as_str()) { return Err("Tipo de imagem não suportado.".to_string()); }
    if input.bytes.len() > 10 * 1024 * 1024 { return Err("A imagem excede o limite de 10 MB.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?;
    let id = Uuid::now_v7().to_string(); let safe_name = input.original_name.replace(['\\', '/', ':'], "_");
    let extension = safe_name.rsplit('.').next().filter(|value| *value != safe_name).unwrap_or("bin");
    let relative = format!("assets/{id}.{extension}"); let path = data_dir(&app)?.join("projects").join(&input.project_id).join(&relative);
    fs::write(&path, &input.bytes).map_err(|error| format!("Não foi possível salvar o asset: {error}"))?;
    let now = timestamp()?;
    connection.execute("INSERT INTO assets(id, original_name, stored_path, media_type, byte_size, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6)", params![id, safe_name, relative, input.media_type, input.bytes.len() as i64, now]).map_err(|error| format!("Não foi possível registrar o asset: {error}"))?;
    Ok(AssetSummary { id, project_id: input.project_id, original_name: safe_name, stored_path: relative, media_type: input.media_type, byte_size: input.bytes.len() as i64, created_at: now })
}

#[tauri::command]
fn read_asset(app: AppHandle, input: ReadAsset) -> Result<AssetBinary, String> {
    let connection = open_project_database(&app, &input.project_id)?;
    let (stored_path, media_type): (String, String) = connection.query_row(
        "SELECT stored_path, media_type FROM assets WHERE id = ?1 AND deleted_at IS NULL",
        params![input.asset_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|error| format!("Não foi possível localizar o asset: {error}"))?;
    let path = data_dir(&app)?.join("projects").join(&input.project_id).join(stored_path);
    let bytes = fs::read(path).map_err(|error| format!("Não foi possível ler o asset: {error}"))?;
    Ok(AssetBinary { media_type, bytes })
}

#[tauri::command]
fn list_notes(app: AppHandle, project_id: String, archived: Option<bool>) -> Result<Vec<NoteSummary>, String> {
    let connection = open_project_database(&app, &project_id)?;
    let archived = archived.unwrap_or(false);
    let mut statement = connection.prepare("SELECT id, project_id, title, content, content_format, is_pinned, is_archived, tags, created_at, updated_at FROM notes WHERE project_id = ?1 AND deleted_at IS NULL AND is_archived = ?2 ORDER BY is_pinned DESC, updated_at DESC").map_err(|error| format!("Não foi possível preparar as Notes: {error}"))?;
    let notes = statement.query_map(params![project_id, archived], read_note).map_err(|error| format!("Não foi possível listar as Notes: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler as Notes: {error}"))?;
    Ok(notes)
}

#[tauri::command]
fn get_note(app: AppHandle, project_id: String, note_id: String) -> Result<NoteSummary, String> {
    let connection = open_project_database(&app, &project_id)?;
    connection.query_row("SELECT id, project_id, title, content, content_format, is_pinned, is_archived, tags, created_at, updated_at FROM notes WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL", params![note_id, project_id], read_note).map_err(|error| format!("Não foi possível abrir a Note: {error}"))
}

#[tauri::command]
fn create_note(app: AppHandle, input: CreateNote) -> Result<NoteSummary, String> {
    let connection = open_project_database(&app, &input.project_id)?;
    let now = timestamp()?; let id = Uuid::now_v7().to_string(); let title = input.title.unwrap_or_else(|| "Untitled note".to_string());
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a criação da Note: {error}"))?;
    transaction.execute("INSERT INTO notes(id, project_id, title, content, content_format, tags, created_at, updated_at) VALUES(?1, ?2, ?3, '', 'markdown', '[]', ?4, ?4)", params![id, input.project_id, title, now]).map_err(|error| format!("Não foi possível criar a Note: {error}"))?;
    append_revision(&transaction, &id, "note", "created", json!({"title": title}), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a Note: {error}"))?;
    get_note(app, input.project_id, id)
}

#[tauri::command]
fn update_note(app: AppHandle, input: UpdateNote) -> Result<NoteSummary, String> {
    if input.title.trim().is_empty() { return Err("O título da Note é obrigatório.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?; let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a atualização da Note: {error}"))?;
    let changed = transaction.execute("UPDATE notes SET title = ?1, content = ?2, tags = ?3, updated_at = ?4 WHERE id = ?5 AND project_id = ?6 AND deleted_at IS NULL", params![input.title.trim(), input.content, serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string()), now, input.note_id, input.project_id]).map_err(|error| format!("Não foi possível salvar a Note: {error}"))?;
    if changed != 1 { return Err("A Note solicitada não existe.".to_string()); }
    append_revision(&transaction, &input.note_id, "note", "updated", json!({"title": input.title.trim()}), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar a Note: {error}"))?;
    get_note(app, input.project_id, input.note_id)
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for character in title.trim().chars() {
        if character.is_alphanumeric() { slug.push(character.to_ascii_lowercase()); last_was_separator = false; }
        else if !last_was_separator && !slug.is_empty() { slug.push('-'); last_was_separator = true; }
    }
    slug.trim_matches('-').to_string()
}

fn ensure_default_doc_space(connection: &Connection, project_id: &str) -> Result<String, String> {
    if let Some(id) = connection.query_row("SELECT id FROM doc_spaces WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY sort_order, created_at LIMIT 1", [project_id], |row| row.get(0)).optional().map_err(|error| format!("Não foi possível abrir o espaço de Docs: {error}"))? { return Ok(id); }
    let id = Uuid::now_v7().to_string(); let now = timestamp()?;
    connection.execute("INSERT INTO doc_spaces(id, project_id, name, sort_order, created_at, updated_at) VALUES(?1, ?2, 'Docs', 0, ?3, ?3)", params![id, project_id, now]).map_err(|error| format!("Não foi possível criar o espaço de Docs: {error}"))?;
    connection.execute("UPDATE docs SET space_id = ?1 WHERE project_id = ?2 AND space_id IS NULL", params![id, project_id]).map_err(|error| format!("Não foi possível organizar os Docs existentes: {error}"))?;
    Ok(id)
}

fn read_doc(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocSummary> {
    Ok(DocSummary { id: row.get(0)?, project_id: row.get(1)?, space_id: row.get(2)?, parent_id: row.get(3)?, kind: row.get(4)?, title: row.get(5)?, slug: row.get(6)?, content: row.get(7)?, content_format: row.get(8)?, sort_order: row.get(9)?, origin: row.get(10)?, created_at: row.get(11)?, updated_at: row.get(12)? })
}

const DOC_SELECT: &str = "SELECT id, project_id, space_id, parent_id, kind, title, slug, content, content_format, sort_order, origin, created_at, updated_at FROM docs";

#[tauri::command]
fn list_docs(app: AppHandle, project_id: String, deleted: Option<bool>) -> Result<Vec<DocSummary>, String> {
    let connection = open_project_database(&app, &project_id)?; ensure_default_doc_space(&connection, &project_id)?;
    let predicate = if deleted.unwrap_or(false) { "deleted_at IS NOT NULL" } else { "deleted_at IS NULL" };
    let query = format!("{DOC_SELECT} WHERE project_id = ?1 AND {predicate} ORDER BY kind DESC, sort_order, updated_at DESC");
    let mut statement = connection.prepare(&query).map_err(|error| format!("Não foi possível preparar os Docs: {error}"))?;
    let docs = statement.query_map([project_id], read_doc).map_err(|error| format!("Não foi possível listar os Docs: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler os Docs: {error}"))?;
    Ok(docs)
}

#[tauri::command]
fn get_doc(app: AppHandle, project_id: String, document_id: String) -> Result<DocSummary, String> {
    let connection = open_project_database(&app, &project_id)?; let query = format!("{DOC_SELECT} WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL");
    connection.query_row(&query, params![document_id, project_id], read_doc).map_err(|error| format!("Não foi possível abrir o Doc: {error}"))
}

#[tauri::command]
fn create_doc(app: AppHandle, input: CreateDoc) -> Result<DocSummary, String> {
    let connection = open_project_database(&app, &input.project_id)?; let kind = input.kind.unwrap_or_else(|| "page".to_string());
    if kind != "page" && kind != "folder" { return Err("O tipo de Doc é inválido.".to_string()); }
    if let Some(parent_id) = &input.parent_id { let valid = connection.query_row("SELECT EXISTS(SELECT 1 FROM docs WHERE id = ?1 AND project_id = ?2 AND kind = 'folder' AND deleted_at IS NULL)", params![parent_id, input.project_id], |row| row.get::<_, bool>(0)).map_err(|error| format!("Não foi possível validar a pasta do Doc: {error}"))?; if !valid { return Err("A pasta selecionada não existe mais.".to_string()); } }
    let title_owned = input.title.unwrap_or_else(|| if kind == "folder" { "Untitled folder".to_string() } else { "Untitled document".to_string() }); let title = title_owned.trim();
    if title.is_empty() { return Err("O título do Doc é obrigatório.".to_string()); }
    let id = Uuid::now_v7().to_string(); let now = timestamp()?; let space_id = ensure_default_doc_space(&connection, &input.project_id)?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a criação do Doc: {error}"))?;
    transaction.execute("INSERT INTO docs(id, project_id, space_id, parent_id, kind, title, slug, content, content_format, sort_order, origin, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, '', 'markdown', 0, 'manual', ?8, ?8)", params![id, input.project_id, space_id, input.parent_id, kind, title, slugify(title), now]).map_err(|error| format!("Não foi possível criar o Doc: {error}"))?;
    append_revision(&transaction, &id, "doc", "created", json!({"title": title}), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar o Doc: {error}"))?;
    get_doc(app, input.project_id, id)
}

#[tauri::command]
fn update_doc(app: AppHandle, input: UpdateDoc) -> Result<DocSummary, String> {
    let title = input.title.trim(); if title.is_empty() { return Err("O título do Doc é obrigatório.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?; let now = timestamp()?;
    let transaction = connection.unchecked_transaction().map_err(|error| format!("Não foi possível iniciar a atualização do Doc: {error}"))?;
    let changed = transaction.execute("UPDATE docs SET title = ?1, slug = ?2, content = ?3, updated_at = ?4 WHERE id = ?5 AND project_id = ?6 AND kind = 'page' AND deleted_at IS NULL", params![title, slugify(title), input.content, now, input.document_id, input.project_id]).map_err(|error| format!("Não foi possível salvar o Doc: {error}"))?;
    if changed != 1 { return Err("O Doc solicitado não existe ou é uma pasta.".to_string()); }
    append_revision(&transaction, &input.document_id, "doc", "updated", json!({"title": title}), now)?;
    transaction.commit().map_err(|error| format!("Não foi possível confirmar o Doc: {error}"))?;
    get_doc(app, input.project_id, input.document_id)
}

#[tauri::command]
fn trash_doc(app: AppHandle, input: DocumentAction) -> Result<(), String> {
    let connection = open_project_database(&app, &input.project_id)?; let now = timestamp()?;
    let changed = connection.execute("WITH RECURSIVE descendants(id) AS (SELECT id FROM docs WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL UNION ALL SELECT docs.id FROM docs JOIN descendants ON docs.parent_id = descendants.id WHERE docs.project_id = ?2 AND docs.deleted_at IS NULL) UPDATE docs SET deleted_at = ?3, updated_at = ?3 WHERE id IN (SELECT id FROM descendants)", params![input.document_id, input.project_id, now]).map_err(|error| format!("Não foi possível mover o Doc para a lixeira: {error}"))?;
    if changed == 0 { return Err("O Doc solicitado não existe.".to_string()); } Ok(())
}

#[tauri::command]
fn restore_doc(app: AppHandle, input: DocumentAction) -> Result<(), String> {
    let connection = open_project_database(&app, &input.project_id)?;
    let changed = connection.execute("UPDATE docs SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND project_id = ?3 AND deleted_at IS NOT NULL", params![timestamp()?, input.document_id, input.project_id]).map_err(|error| format!("Não foi possível restaurar o Doc: {error}"))?;
    if changed != 1 { return Err("O Doc solicitado não está na lixeira.".to_string()); } Ok(())
}

#[tauri::command]
fn export_doc_markdown(app: AppHandle, input: ExportDocMarkdown) -> Result<(), String> {
    let path = PathBuf::from(&input.destination_path);
    if path.extension().and_then(|extension| extension.to_str()).map(|extension| extension.eq_ignore_ascii_case("md")) != Some(true) { return Err("Escolha um arquivo com extensão .md.".to_string()); }
    let doc = get_doc(app, input.project_id, input.document_id)?; if doc.kind != "page" { return Err("Somente documentos podem ser exportados.".to_string()); }
    fs::write(path, doc.content).map_err(|error| format!("Não foi possível exportar o Markdown: {error}"))
}

fn secret_entry(name: &str) -> Result<keyring::Entry, String> { keyring::Entry::new("Orbit", name).map_err(|error| format!("Não foi possível acessar o cofre do sistema: {error}")) }
fn has_secret(name: &str) -> bool { secret_entry(name).and_then(|entry| entry.get_password().map_err(|error| error.to_string())).map(|value| !value.is_empty()).unwrap_or(false) }
fn get_secret(name: &str) -> Result<String, String> { secret_entry(name)?.get_password().map_err(|_| "A credencial não está disponível no cofre do sistema.".to_string()) }

#[tauri::command]
fn get_ai_settings(app: AppHandle) -> Result<AiSettings, String> {
    let catalog = open_catalog(&app)?;
    let (enabled, provider, model): (bool, String, String) = catalog.query_row("SELECT enabled, provider, model FROM ai_settings WHERE id = 1", [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).map_err(|error| format!("Não foi possível ler as configurações de IA: {error}"))?;
    Ok(AiSettings { enabled, provider, model, has_openai_api_key: has_secret("openai-api-key"), has_github_token: has_secret("github-token") })
}

#[tauri::command]
fn save_ai_settings(app: AppHandle, input: SaveAiSettings) -> Result<AiSettings, String> {
    if input.model.trim().is_empty() { return Err("Informe um modelo da OpenAI.".to_string()); }
    let catalog = open_catalog(&app)?; catalog.execute("UPDATE ai_settings SET enabled = ?1, model = ?2, updated_at = ?3 WHERE id = 1", params![input.enabled, input.model.trim(), timestamp()?]).map_err(|error| format!("Não foi possível salvar as configurações de IA: {error}"))?;
    get_ai_settings(app)
}

#[tauri::command]
fn save_openai_api_key(input: SaveSecret) -> Result<(), String> { if input.value.trim().is_empty() { return Err("A chave não pode ficar vazia.".to_string()); } secret_entry("openai-api-key")?.set_password(input.value.trim()).map_err(|error| format!("Não foi possível salvar a chave no cofre do sistema: {error}")) }
#[tauri::command]
fn save_github_token(input: SaveSecret) -> Result<(), String> { if input.value.trim().is_empty() { return Err("O token não pode ficar vazio.".to_string()); } secret_entry("github-token")?.set_password(input.value.trim()).map_err(|error| format!("Não foi possível salvar o token no cofre do sistema: {error}")) }
#[tauri::command]
fn delete_openai_api_key() -> Result<(), String> { secret_entry("openai-api-key")?.delete_credential().map_err(|error| format!("Não foi possível remover a chave do cofre: {error}")) }

fn source_path_is_blocked(path: &Path) -> bool {
    path.components().any(|component| match component {
        std::path::Component::Normal(value) => matches!(value.to_string_lossy().to_ascii_lowercase().as_str(), "node_modules" | ".git" | "target" | "dist" | "build" | ".next"),
        _ => false,
    })
}

fn allowed_source(path: &Path) -> bool {
    if source_path_is_blocked(path) { return false; }
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else { return false; };
    let lower = name.to_ascii_lowercase();
    if lower == ".env" || lower.ends_with(".pem") || lower.ends_with(".key") || lower.ends_with(".pfx") || lower.ends_with(".p12") { return false; }
    [".md", ".txt", ".json", ".toml", ".yaml", ".yml", ".ts", ".tsx", ".js", ".jsx", ".rs", ".py", ".go", ".java", ".cs", ".sql", ".env.example"].iter().any(|extension| lower.ends_with(extension))
}
fn local_sources(path: &str, folder: bool) -> Result<Vec<Value>, String> {
    let paths: Vec<PathBuf> = if folder { walkdir::WalkDir::new(path).into_iter().filter_entry(|entry| !source_path_is_blocked(entry.path())).filter_map(Result::ok).filter(|entry| entry.file_type().is_file()).map(|entry| entry.path().to_path_buf()).collect() } else { vec![PathBuf::from(path)] };
    let mut sources = Vec::new(); let mut total = 0usize;
    for path in paths.into_iter().filter(|path| allowed_source(path)).take(24) { let bytes = fs::read(&path).map_err(|error| format!("Não foi possível ler a fonte local: {error}"))?; if bytes.len() > 60_000 || total + bytes.len() > 300_000 { continue; } let text = String::from_utf8(bytes).map_err(|_| "Esta versão aceita arquivos de texto, Markdown e código. PDF e DOCX serão adicionados com o extrator local.".to_string())?; total += text.len(); sources.push(json!({"id":Uuid::now_v7().to_string(),"kind":"uploaded_document","path":path.to_string_lossy(),"selectedReason":"arquivo local selecionado","content":text})); }
    if sources.is_empty() { return Err("Nenhum arquivo textual permitido foi encontrado.".to_string()); } Ok(sources)
}
fn github_sources(url: &str) -> Result<Vec<Value>, String> {
    let trimmed = url.trim().trim_end_matches('/'); let parts: Vec<_> = trimmed.split('/').collect(); if parts.len() < 5 || parts[2] != "github.com" { return Err("Use uma URL de repositório GitHub válida.".to_string()); }
    let repo = format!("{}/{}", parts[3], parts[4]); let client = reqwest::blocking::Client::builder().user_agent("Orbit Desktop").build().map_err(|error| format!("Não foi possível preparar GitHub: {error}"))?;
    let mut request = client.get(format!("https://api.github.com/repos/{repo}/git/trees/HEAD?recursive=1")); if let Ok(token) = get_secret("github-token") { request = request.bearer_auth(token); }
    let payload: Value = request.send().map_err(|error| format!("Não foi possível acessar o GitHub: {error}"))?.error_for_status().map_err(|error| format!("GitHub recusou o acesso: {error}"))?.json().map_err(|error| format!("Resposta inválida do GitHub: {error}"))?;
    let priority = ["readme", "package.json", "docker-compose", "compose.", "cargo.toml", "requirements.txt", ".github/workflows"];
    let mut files: Vec<String> = payload.get("tree").and_then(Value::as_array).into_iter().flatten().filter_map(|entry| entry.get("path").and_then(Value::as_str)).filter(|path| allowed_source(Path::new(path))).map(ToString::to_string).collect(); files.sort_by_key(|path| if priority.iter().any(|item| path.to_lowercase().contains(item)) { 0 } else { 1 });
    let mut sources = Vec::new(); let mut total = 0usize; for path in files.into_iter().take(24) { let mut request = client.get(format!("https://raw.githubusercontent.com/{repo}/HEAD/{path}")); if let Ok(token) = get_secret("github-token") { request = request.bearer_auth(token); } let response = request.send().ok().and_then(|response| response.error_for_status().ok()); if let Some(response) = response { let text = response.text().unwrap_or_default(); if text.len() <= 60_000 && total + text.len() <= 300_000 { total += text.len(); sources.push(json!({"id":Uuid::now_v7().to_string(),"kind":"repository_file","path":path,"selectedReason":"arquivo priorizado do repositório","content":text})); } } }
    if sources.is_empty() { return Err("Não foi possível ler arquivos permitidos desse repositório.".to_string()); } Ok(sources)
}
fn architecture_schema() -> Value { json!({"type":"object","additionalProperties":false,"properties":{"project":{"type":"object","additionalProperties":false,"properties":{"name":{"type":"string"},"summary":{"type":"string"},"sourceMode":{"type":"string"}},"required":["name","summary","sourceMode"]},"nodes":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"id":{"type":"string"},"kind":{"type":"string"},"name":{"type":"string"},"technology":{"type":"array","items":{"type":"string"}},"confidence":{"type":"number"},"evidence":{"type":"array","items":{"type":"string"}}},"required":["id","kind","name","technology","confidence","evidence"]}},"edges":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"id":{"type":"string"},"sourceNodeId":{"type":"string"},"targetNodeId":{"type":"string"},"protocol":{"type":"string"},"relationship":{"type":"string"},"confidence":{"type":"number"}},"required":["id","sourceNodeId","targetNodeId","protocol","relationship","confidence"]}},"assumptions":{"type":"array","items":{"type":"string"}},"documentationOutline":{"type":"array","items":{"type":"string"}}},"required":["project","nodes","edges","assumptions","documentationOutline"]}) }
fn response_text(value: &Value) -> Option<String> { value.get("output")?.as_array()?.iter().find_map(|item| item.get("content")?.as_array()?.iter().find_map(|part| part.get("text").and_then(Value::as_str).map(ToString::to_string))) }
fn markdown_from_model(model: &Value) -> String { let project = model.get("project").unwrap_or(&Value::Null); let mut output = format!("# {}\n\n{}\n\n## Arquitetura\n", project.get("name").and_then(Value::as_str).unwrap_or("Projeto"), project.get("summary").and_then(Value::as_str).unwrap_or("")); for node in model.get("nodes").and_then(Value::as_array).into_iter().flatten() { output.push_str(&format!("\n### {}\n{} · confiança {}\n", node.get("name").and_then(Value::as_str).unwrap_or("Componente"), node.get("kind").and_then(Value::as_str).unwrap_or("unknown"), node.get("confidence").unwrap_or(&Value::Null))); } output }

#[tauri::command]
fn start_architecture_analysis(app: AppHandle, input: StartArchitectureAnalysis) -> Result<ArchitectureGeneration, String> {
    if !input.consent_to_send_sources { return Err("Confirme que as fontes selecionadas podem ser enviadas ao provedor de IA.".to_string()); }
    let settings = get_ai_settings(app.clone())?; if !settings.enabled { return Err("A IA está desabilitada nas configurações.".to_string()); } let api_key = get_secret("openai-api-key")?;
    let mut sources = Vec::new(); if let Some(url) = input.repository_url.as_deref().filter(|value| !value.trim().is_empty()) { sources.extend(github_sources(url)?); } if let Some(path) = input.document_path.as_deref().filter(|value| !value.trim().is_empty()) { sources.extend(local_sources(path, false)?); } if let Some(path) = input.folder_path.as_deref().filter(|value| !value.trim().is_empty()) { sources.extend(local_sources(path, true)?); }
    if sources.is_empty() { return Err("Selecione um repositório, documento ou pasta antes de analisar.".to_string()); }
    let source_mode = if input.repository_url.is_some() && (input.document_path.is_some() || input.folder_path.is_some()) { "repository_plus_document" } else if input.repository_url.is_some() { "repository" } else { "document" }.to_string(); let source_payload: Vec<Value> = sources.iter().map(|source| json!({"id":source["id"],"kind":source["kind"],"path":source["path"],"selectedReason":source["selectedReason"]})).collect();
    let prompt_sources: Vec<Value> = sources.iter().map(|source| json!({"path":source["path"],"content":source["content"]})).collect();
    let body = json!({"model":settings.model,"input":[{"role":"system","content":"Analyze only supplied evidence. Return architecture facts with confidence 0 to 1. Mark uncertainty as assumptions. Do not invent services."},{"role":"user","content":format!("Source mode: {source_mode}. Sources JSON:\n{}", serde_json::to_string(&prompt_sources).unwrap_or_default())}],"text":{"format":{"type":"json_schema","name":"architecture_model","strict":true,"schema":architecture_schema()}}});
    let value: Value = reqwest::blocking::Client::new().post("https://api.openai.com/v1/responses").bearer_auth(api_key).json(&body).send().map_err(|error| format!("Não foi possível chamar a OpenAI: {error}"))?.error_for_status().map_err(|error| format!("A OpenAI recusou a análise: {error}"))?.json().map_err(|error| format!("Resposta inválida da OpenAI: {error}"))?;
    let model: Value = serde_json::from_str(&response_text(&value).ok_or_else(|| "A OpenAI não devolveu um resultado estruturado.".to_string())?).map_err(|_| "A OpenAI devolveu um modelo de arquitetura inválido.".to_string())?; if !model.get("nodes").map(Value::is_array).unwrap_or(false) || !model.get("edges").map(Value::is_array).unwrap_or(false) { return Err("O modelo de arquitetura não passou na validação local.".to_string()); }
    let id = Uuid::now_v7().to_string(); let now = timestamp()?; let documentation_markdown = markdown_from_model(&model); let connection = open_project_database(&app, &input.project_id)?;
    connection.execute("INSERT INTO ai_generations(id, project_id, status, source_mode, sources, architecture_model, documentation_markdown, created_at, updated_at) VALUES(?1, ?2, 'ready_for_review', ?3, ?4, ?5, ?6, ?7, ?7)", params![id, input.project_id, source_mode, serde_json::to_string(&source_payload).unwrap_or_default(), serde_json::to_string(&model).unwrap_or_default(), documentation_markdown, now]).map_err(|error| format!("Não foi possível salvar o rascunho de IA: {error}"))?;
    Ok(ArchitectureGeneration { id, project_id: input.project_id, status:"ready_for_review".to_string(), source_mode, sources:Value::Array(source_payload), architecture_model:Some(model), documentation_markdown:Some(documentation_markdown), error:None, created_at:now, updated_at:now })
}

#[tauri::command]
fn save_simulation_run(app: AppHandle, input: SaveSimulationRun) -> Result<SimulationRunSummary, String> {
    if input.canvas_id.trim().is_empty() || input.scenario.trim().is_empty() { return Err("Canvas e cenário são obrigatórios para salvar a simulação.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?;
    let id = Uuid::now_v7().to_string();
    let created_at = timestamp()?;
    connection.execute(
        "INSERT INTO simulation_runs(id, project_id, canvas_id, scenario, seed, config, canvas_snapshot, result, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, input.project_id, input.canvas_id, input.scenario.trim(), input.seed, serde_json::to_string(&input.config).map_err(|error| format!("Não foi possível serializar a configuração: {error}"))?, serde_json::to_string(&input.canvas_snapshot).map_err(|error| format!("Não foi possível serializar o snapshot: {error}"))?, serde_json::to_string(&input.result).map_err(|error| format!("Não foi possível serializar o resultado: {error}"))?, created_at],
    ).map_err(|error| format!("Não foi possível salvar a execução local: {error}"))?;
    Ok(SimulationRunSummary { id, created_at })
}

fn valid_calendar_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' { return false; }
    let parts: Vec<_> = value.split('-').collect();
    let year = parts.first().and_then(|part| part.parse::<u32>().ok());
    let month = parts.get(1).and_then(|part| part.parse::<u32>().ok());
    let day = parts.get(2).and_then(|part| part.parse::<u32>().ok());
    matches!((year, month, day), (Some(year), Some(month), Some(day)) if year > 0 && (1..=12).contains(&month) && (1..=31).contains(&day))
}

fn read_calendar_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarEntry> {
    Ok(CalendarEntry { id: row.get(0)?, project_id: row.get(1)?, entry_date: row.get(2)?, content: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
}

#[tauri::command]
fn list_calendar_entries(app: AppHandle, project_id: String, start_date: String, end_date: String) -> Result<Vec<CalendarEntry>, String> {
    if !valid_calendar_date(&start_date) || !valid_calendar_date(&end_date) || start_date > end_date { return Err("Informe um intervalo de datas válido para o Calendar.".to_string()); }
    let connection = open_project_database(&app, &project_id)?;
    let mut statement = connection.prepare("SELECT id, project_id, entry_date, content, created_at, updated_at FROM calendar_entries WHERE project_id = ?1 AND entry_date >= ?2 AND entry_date <= ?3 AND deleted_at IS NULL ORDER BY entry_date, created_at")
        .map_err(|error| format!("Não foi possível preparar o Calendar: {error}"))?;
    let entries = statement.query_map(params![project_id, start_date, end_date], read_calendar_entry)
        .map_err(|error| format!("Não foi possível listar o Calendar: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Não foi possível ler o Calendar: {error}"))?;
    Ok(entries)
}

#[tauri::command]
fn create_calendar_entry(app: AppHandle, input: CreateCalendarEntry) -> Result<CalendarEntry, String> {
    if !valid_calendar_date(&input.entry_date) { return Err("Escolha uma data válida para o planejamento.".to_string()); }
    let content = input.content.trim();
    if content.is_empty() { return Err("Escreva o que você quer planejar para este dia.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?;
    let now = timestamp()?;
    let id = Uuid::now_v7().to_string();
    connection.execute("INSERT INTO calendar_entries(id, project_id, entry_date, content, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?5)", params![id, input.project_id, input.entry_date, content, now])
        .map_err(|error| format!("Não foi possível salvar o planejamento: {error}"))?;
    Ok(CalendarEntry { id, project_id: input.project_id, entry_date: input.entry_date, content: content.to_string(), created_at: now, updated_at: now })
}

#[tauri::command]
fn archive_note(app: AppHandle, project_id: String, note_id: String, archived: bool) -> Result<(), String> {
    let connection = open_project_database(&app, &project_id)?;
    connection.execute("UPDATE notes SET is_archived = ?1, updated_at = ?2 WHERE id = ?3 AND project_id = ?4 AND deleted_at IS NULL", params![archived, timestamp()?, note_id, project_id]).map_err(|error| format!("Não foi possível arquivar a Note: {error}"))?;
    Ok(())
}

#[tauri::command]
fn toggle_note_pin(app: AppHandle, project_id: String, note_id: String) -> Result<(), String> {
    let connection = open_project_database(&app, &project_id)?;
    connection.execute("UPDATE notes SET is_pinned = CASE is_pinned WHEN 1 THEN 0 ELSE 1 END, updated_at = ?1 WHERE id = ?2 AND project_id = ?3 AND deleted_at IS NULL", params![timestamp()?, note_id, project_id]).map_err(|error| format!("Não foi possível fixar a Note: {error}"))?;
    Ok(())
}

#[tauri::command]
fn search_notes(app: AppHandle, project_id: String, query: String, archived: Option<bool>) -> Result<Vec<NoteSummary>, String> {
    let connection = open_project_database(&app, &project_id)?; let pattern = format!("%{}%", query.trim());
    let mut statement = connection.prepare("SELECT id, project_id, title, content, content_format, is_pinned, is_archived, tags, created_at, updated_at FROM notes WHERE project_id = ?1 AND deleted_at IS NULL AND is_archived = ?2 AND (title LIKE ?3 OR content LIKE ?3) ORDER BY is_pinned DESC, updated_at DESC").map_err(|error| format!("Não foi possível preparar a busca das Notes: {error}"))?;
    let notes = statement.query_map(params![project_id, archived.unwrap_or(false), pattern], read_note).map_err(|error| format!("Não foi possível buscar as Notes: {error}"))?.collect::<Result<Vec<_>, _>>().map_err(|error| format!("Não foi possível ler a busca das Notes: {error}"))?;
    Ok(notes)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = focus_main_window(window);
            }
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_local_profile,
            get_local_profile,
            update_local_profile,
            replace_profile_asset,
            remove_profile_asset,
            read_profile_asset,
            get_profile_overview,
            list_profile_resources,
            create_local_project,
            trash_local_project,
            list_local_projects,
            list_templates,
            set_template_favorite,
            create_project_from_template,
            apply_template_to_project,
            list_personal_library,
            add_library_component,
            load_canvas,
            create_canvas_component,
            create_canvas_visual,
            update_canvas_visual,
            delete_canvas_visual,
            move_canvas_element,
            update_canvas_component,
            delete_canvas_component,
            create_canvas_connection,
            list_calendar_entries,
            create_calendar_entry,
            list_notes,
            create_note,
            get_note,
            update_note,
            archive_note,
            toggle_note_pin,
            search_notes,
            list_docs,
            create_doc,
            get_doc,
            update_doc,
            trash_doc,
            restore_doc,
            export_doc_markdown,
            get_ai_settings,
            save_ai_settings,
            save_openai_api_key,
            save_github_token,
            delete_openai_api_key,
            start_architecture_analysis,
            save_simulation_run,
            create_asset,
            read_asset
        ])
        .run(tauri::generate_context!())
        .expect("error while running Orbit");
}

fn focus_main_window(window: WebviewWindow) -> Result<(), tauri::Error> {
    window.show()?;
    window.set_focus()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_database_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("orbit-{name}-{}.db", Uuid::now_v7()))
    }

    fn table_exists(connection: &Connection, table: &str) -> bool {
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                [table],
                |row| row.get(0),
            )
            .expect("table lookup should succeed")
    }

    #[test]
    fn catalog_migration_keeps_canvas_schema_out_of_catalog() {
        let connection = Connection::open_in_memory().expect("catalog should open");

        migrate_catalog(&connection).expect("catalog migration should succeed");

        assert!(table_exists(&connection, "projects"));
        assert!(table_exists(&connection, "template_favorites"));
        assert!(!table_exists(&connection, "canvases"));
        assert!(!table_exists(&connection, "canvas_elements"));
    }

    #[test]
    fn catalog_migration_creates_profile_schema() {
        let connection = Connection::open_in_memory().expect("catalog should open");
        migrate_catalog(&connection).expect("catalog migration should succeed");
        assert!(table_exists(&connection, "profile_details"));
        assert!(table_exists(&connection, "profile_assets"));
        assert!(table_exists(&connection, "profile_technologies"));
        assert!(connection.query_row("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 7)", [], |row| row.get::<_, bool>(0)).expect("profile migration should be registered"));
    }

    #[test]
    fn existing_project_database_receives_canvas_migration() {
        let path = temporary_database_path("project-migration");
        let connection = Connection::open(&path).expect("project database should open");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
                 INSERT INTO schema_migrations(version, applied_at) VALUES(1, 0);
                 CREATE TABLE project_metadata (project_id TEXT PRIMARY KEY NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
                 CREATE TABLE assets (id TEXT PRIMARY KEY NOT NULL, original_name TEXT NOT NULL, stored_path TEXT NOT NULL UNIQUE, media_type TEXT NOT NULL, byte_size INTEGER NOT NULL CHECK(byte_size >= 0), created_at INTEGER NOT NULL, deleted_at INTEGER);
                 CREATE TABLE revisions (id TEXT PRIMARY KEY NOT NULL, entity_id TEXT NOT NULL, entity_type TEXT NOT NULL, operation TEXT NOT NULL, payload TEXT NOT NULL, created_at INTEGER NOT NULL);",
            )
            .expect("legacy project schema should be created");
        drop(connection);

        migrate_project_database(&path).expect("project migration should succeed");

        let migrated = Connection::open(&path).expect("migrated project should open");
        assert!(table_exists(&migrated, "canvases"));
        assert!(table_exists(&migrated, "canvas_elements"));
        assert!(table_exists(&migrated, "components"));
        assert!(table_exists(&migrated, "component_ports"));
        assert!(table_exists(&migrated, "connections"));
        assert!(table_exists(&migrated, "notes"));
        assert!(table_exists(&migrated, "docs"));
        assert!(table_exists(&migrated, "doc_spaces"));
        assert!(table_exists(&migrated, "calendar_entries"));
        assert!(table_exists(&migrated, "simulation_runs"));
        let metadata_columns = migrated.prepare("PRAGMA table_info(project_metadata)").expect("metadata schema should be readable")
            .query_map([], |row| row.get::<_, String>(1)).expect("metadata columns should be readable")
            .collect::<Result<Vec<_>, _>>().expect("metadata columns should load");
        assert!(metadata_columns.contains(&"template_id".to_string()));
        assert!(metadata_columns.contains(&"template_version".to_string()));
        let docs_columns = migrated.prepare("PRAGMA table_info(docs)").expect("docs schema should be readable")
            .query_map([], |row| row.get::<_, String>(1)).expect("docs columns should be readable")
            .collect::<Result<Vec<_>, _>>().expect("docs columns should load");
        assert!(docs_columns.contains(&"space_id".to_string()));
        assert!(docs_columns.contains(&"parent_id".to_string()));
        assert!(docs_columns.contains(&"kind".to_string()));
        assert!(migrated
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("migration lookup should succeed"));
        assert!(migrated
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 3)",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("notes migration lookup should succeed"));
        assert!(migrated
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("template project migration lookup should succeed"));
        assert!(migrated
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 5)",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("calendar migration lookup should succeed"));
        assert!(migrated
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 6)",
                [],
                |row| row.get::<_, bool>(0),
            )
            .expect("docs migration lookup should succeed"));

        drop(migrated);
        fs::remove_file(path).expect("temporary project database should be removed");
    }

    #[test]
    fn local_project_directory_only_accepts_its_own_project_folder() {
        let root = Path::new("orbit-data");
        assert_eq!(
            local_project_directory(root, "projects/project-1", "project-1").expect("own folder should be valid"),
            root.join("projects").join("project-1"),
        );
        assert!(local_project_directory(root, "projects/../other", "project-1").is_err());
        assert!(local_project_directory(root, "assets/project-1", "project-1").is_err());
    }

    #[test]
    fn aws_template_persists_the_complete_vertical_topology() {
        let path = temporary_database_path("aws-template");
        migrate_project_database(&path).expect("template database should migrate");
        let mut connection = Connection::open(&path).expect("template database should open");
        connection.execute_batch("PRAGMA foreign_keys = ON;").expect("foreign keys should be enabled");
        connection.execute("INSERT INTO project_metadata(project_id, created_at, updated_at) VALUES('project-aws', 1, 1)", [])
            .expect("project metadata should exist");
        let input = |key: &str| TemplatePortSeed { key: key.to_string(), direction: "input".to_string(), protocol: "data".to_string(), data: Value::Null };
        let output = |key: &str| TemplatePortSeed { key: key.to_string(), direction: "output".to_string(), protocol: "data".to_string(), data: Value::Null };
        let component = |component_type: &str, label: &str, x: f64, y: f64, ports: Vec<TemplatePortSeed>| TemplateComponentSeed { component_type: component_type.to_string(), label: label.to_string(), description: "AWS service".to_string(), x, y, width: 180.0, height: 92.0, color: "blue".to_string(), ports, data: Value::Null };
        let seed = TemplateSeed {
            version: "1.2.0".to_string(),
            canvas_name: "AWS Infrastructure".to_string(),
            components: vec![
                component("aws-route-53", "Route 53", 460.0, 40.0, vec![output("http-out")]),
                component("aws-cloudfront", "CloudFront", 100.0, 250.0, vec![input("http-in")]),
                component("aws-s3", "S3 (Static)", 460.0, 250.0, vec![input("http-in"), output("data-out")]),
                component("aws-api-gateway", "API Gateway", 820.0, 250.0, vec![input("http-in")]),
                component("aws-lambda", "Lambda", 100.0, 500.0, vec![input("data-in")]),
                component("aws-ec2", "EC2", 460.0, 500.0, vec![input("data-in")]),
                component("aws-dynamodb", "DynamoDB", 820.0, 500.0, vec![input("data-in")]),
            ],
            connections: vec![
                TemplateConnectionSeed { source_index: 0, source_port_key: "http-out".to_string(), target_index: 1, target_port_key: "http-in".to_string(), connection_type: "http".to_string() },
                TemplateConnectionSeed { source_index: 0, source_port_key: "http-out".to_string(), target_index: 2, target_port_key: "http-in".to_string(), connection_type: "http".to_string() },
                TemplateConnectionSeed { source_index: 0, source_port_key: "http-out".to_string(), target_index: 3, target_port_key: "http-in".to_string(), connection_type: "http".to_string() },
                TemplateConnectionSeed { source_index: 2, source_port_key: "data-out".to_string(), target_index: 4, target_port_key: "data-in".to_string(), connection_type: "data".to_string() },
                TemplateConnectionSeed { source_index: 2, source_port_key: "data-out".to_string(), target_index: 5, target_port_key: "data-in".to_string(), connection_type: "data".to_string() },
                TemplateConnectionSeed { source_index: 2, source_port_key: "data-out".to_string(), target_index: 6, target_port_key: "data-in".to_string(), connection_type: "data".to_string() },
            ],
            notes: vec![], docs: vec![],
        };

        let canvas_id = persist_template_content(&mut connection, "project-aws", "aws-infrastructure", &seed, 2)
            .expect("AWS template should persist");
        let snapshot = load_canvas_snapshot(&mut connection, "project-aws", Some(&canvas_id))
            .expect("AWS template canvas should load");

        assert_eq!(snapshot.components.len(), 7);
        assert_eq!(snapshot.connections.len(), 6);
        assert_eq!(snapshot.components[0].label, "Route 53");
        assert_eq!(snapshot.components[0].y, 40.0);
        assert_eq!(snapshot.components[3].label, "API Gateway");
        assert_eq!(snapshot.components[6].label, "DynamoDB");
        assert_eq!(snapshot.components[6].y, 500.0);
        assert!(snapshot.components.iter().flat_map(|component| component.ports.iter()).all(|port| port.data.is_object()));

        drop(connection);
        fs::remove_file(path).expect("template database should be removed");
    }
}
