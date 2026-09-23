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
    migrate_component_library_catalog(connection)
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
    migrate_notes_database(&connection)
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
) -> Result<CanvasSnapshot, String> {
    let canvas = ensure_canvas(connection, project_id)?;
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
fn load_canvas(app: AppHandle, project_id: String) -> Result<CanvasSnapshot, String> {
    let mut connection = open_project_database(&app, &project_id)?;
    load_canvas_snapshot(&mut connection, &project_id)
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
    load_canvas_snapshot(&mut connection, &input.project_id)
}

#[tauri::command]
fn create_canvas_visual(app: AppHandle, input: CreateCanvasVisual) -> Result<CanvasSnapshot, String> {
    if input.visual_type != "image" { return Err("Somente imagens visuais são suportadas nesta etapa.".to_string()); }
    let mut connection = open_project_database(&app, &input.project_id)?;
    assert_canvas(&connection, &input.canvas_id, &input.project_id)?;
    let now = timestamp()?; let id = Uuid::now_v7().to_string();
    connection.execute("INSERT INTO canvas_elements(id, canvas_id, kind, type, x, y, width, height, data, created_at, updated_at) VALUES(?1, ?2, 'visual', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)", params![id, input.canvas_id, input.visual_type, input.x, input.y, input.width.max(80.0), input.height.max(60.0), input.data.to_string(), now]).map_err(|error| format!("Não foi possível criar a imagem no Canvas: {error}"))?;
    load_canvas_snapshot(&mut connection, &input.project_id)
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
    load_canvas_snapshot(&mut connection, &input.project_id)
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
    load_canvas_snapshot(&mut connection, &input.project_id)
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
    load_canvas_snapshot(&mut connection, &input.project_id)
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
    connection.execute("INSERT INTO notes(id, project_id, title, content, content_format, tags, created_at, updated_at) VALUES(?1, ?2, ?3, '', 'markdown', '[]', ?4, ?4)", params![id, input.project_id, title, now]).map_err(|error| format!("Não foi possível criar a Note: {error}"))?;
    get_note(app, input.project_id, id)
}

#[tauri::command]
fn update_note(app: AppHandle, input: UpdateNote) -> Result<NoteSummary, String> {
    if input.title.trim().is_empty() { return Err("O título da Note é obrigatório.".to_string()); }
    let connection = open_project_database(&app, &input.project_id)?; let now = timestamp()?;
    let changed = connection.execute("UPDATE notes SET title = ?1, content = ?2, tags = ?3, updated_at = ?4 WHERE id = ?5 AND project_id = ?6 AND deleted_at IS NULL", params![input.title.trim(), input.content, serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string()), now, input.note_id, input.project_id]).map_err(|error| format!("Não foi possível salvar a Note: {error}"))?;
    if changed != 1 { return Err("A Note solicitada não existe.".to_string()); }
    get_note(app, input.project_id, input.note_id)
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
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = focus_main_window(window);
            }
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
            initialize_local_profile,
            create_local_project,
            list_local_projects,
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
            list_notes,
            create_note,
            get_note,
            update_note,
            archive_note,
            toggle_note_pin,
            search_notes,
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
        assert!(!table_exists(&connection, "canvases"));
        assert!(!table_exists(&connection, "canvas_elements"));
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

        drop(migrated);
        fs::remove_file(path).expect("temporary project database should be removed");
    }
}
