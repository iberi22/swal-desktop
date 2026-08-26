# Plan 100% — Completar swal-desktop (Estable, Portable, Seguro, Zero-Eww)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.
> Cada tarea es independiente y commiteable. Orden estricto por fases.

**Goal:** Llevar swal-desktop de ~90% a 100%: compilación limpia, cero inyección de comandos,
cero paths hardcodeados, migración Eww→Rust completa, CI con gates de seguridad y push a repo público saneado.

**Architecture:** Workspace Rust de 8 crates + capa legacy Eww en coexistencia. El plan elimina
la capa legacy, hace portables los bins/servicios y añade CI que impide regresiones de seguridad.

**Tech Stack:** Rust (tokio, wgpu, serde), NixOS flake, systemd user units, Python (scripts Eww restantes), GitHub Actions.

**Estado verificado hoy (evidencia real):**
- `cargo test --workspace` FALLA: `E0063` en `native_window.rs:179`, `test_full_core_coverage.rs:201`, `test_standalone_crossplatform_e2e.rs:367` (faltan `path_filter_memory` y `saved_filter_presets` en inicializadores de `SessionState`) + errores de linking gcc en swal-files.
- 32 ocurrencias de `/home/belal` en `.rs`; también en `nixos/swal-node.nix`.
- Command injection: `eww/scripts/hermes_orb_menu.py` usa `subprocess.Popen(cmd, shell=True)` con input del usuario (4 sitios).
- Socket con UID fijo `/run/user/1000/swal/telemetry.sock` en `crates/swal-node-daemon/src/main.rs`.
- Daemon Rust aún llama a scripts Eww (`toggle_dashboard.sh`, `toggle_orb_hud.sh`) como fallback.
- Sin CI (no existe `.github/workflows/`).
- 1 commit local sin pushear.
- Warnings menores: unused imports/vars en a2ui-engine, ambient-orb, node-daemon; dead code en cli.rs/settings_cli.rs.

---

## FASE 0 — Desbloquear compilación (P0, bloquea todo)

### Task 0.1: Arreglar E0063 en native_window.rs

**Objective:** `SessionState` en el test de `native_window.rs` incluye los 2 campos nuevos.

**Files:**
- Modify: `crates/swal-files/src/native_window.rs:179-200`

**Step 1: Añadir campos al inicializador del test**

Dentro del literal `SessionState { ... }` (línea ~179), después de `selected_path: None,` añadir:

```rust
            selected_path: None,
            saved_filter_presets: Vec::new(),
            path_filter_memory: std::collections::HashMap::new(),
```

**Step 2: Verificar**

Run: `cd crates/swal-files && cargo check --lib`
Expected: `Finished` sin errores E0063.

**Step 3: Commit**

```bash
git add crates/swal-files/src/native_window.rs
git commit -m "fix(swal-files): initialize new SessionState filter fields in native_window test"
```

### Task 0.2: Arreglar E0063 en tests E2E

**Objective:** Mismo fix en los 2 tests de integración.

**Files:**
- Modify: `crates/swal-files/tests/test_full_core_coverage.rs:201`
- Modify: `crates/swal-files/tests/test_standalone_crossplatform_e2e.rs:367`

**Step 1:** En cada literal `SessionState { ... }` añadir:

```rust
    saved_filter_presets: Vec::new(),
    path_filter_memory: std::collections::HashMap::new(),
```

**Step 2: Verificar**

Run: `cargo test -p swal-files 2>&1 | tail -20`
Expected: compila; si quedan fallos de LINKING (gcc), capturar el símbolo no encontrado para Task 0.3.

**Step 3: Commit**

```bash
git add crates/swal-files/tests/
git commit -m "fix(swal-files): add missing SessionState fields in integration tests"
```

### Task 0.3: Diagnosticar y arreglar errores de linking gcc en swal-files

**Objective:** `cargo build -p swal-files` termina sin `linking with gcc failed`.

**Files:**
- Modify: `crates/swal-files/Cargo.toml` (según diagnóstico)

**Step 1: Obtener el error completo**

Run: `cargo build -p swal-files --tests 2>&1 | grep -B5 "undefined reference" | head -30`

**Step 2: Causas probables y fix**
- Si falta una dep nativa (p.ej. `gtk`/`wayland-sys` sin feature): ajustar features en Cargo.toml o envolver el módulo en `#[cfg(feature = "gui")]`.
- Si es símbolo duplicado de libc::kill: mover el uso tras `#[cfg(target_os = "linux")]`.

**Step 3: Verificar workspace completo**

Run: `cargo test --workspace 2>&1 | grep "test result"`
Expected: `28+ passed; 0 failed` (README declara 28).

**Step 4: Commit**

```bash
git add -A
git commit -m "fix(build): resolve swal-files linker errors, full workspace green"
```

---

## FASE 1 — Seguridad P0 (sin esto NO se publica ni se recomienda instalar)

### Task 1.1: Eliminar command injection en hermes_orb_menu.py

**Objective:** Ningún input llega a shell=True.

**Files:**
- Modify: `eww/scripts/hermes_orb_menu.py:50-70`

**Step 1: Reemplazar dispatch_action**

```python
def dispatch_action(action: str, extra_args: str = ""):
    payload = {
        "event": "action_triggered",
        "action": action,
        "extra_args": extra_args,
        "timestamp": time.time(),
    }
    socket_sent = send_unix_socket_payload(payload)

    prompts = {
        "@summarize": "Resumir el contexto activo o selección de texto",
        "@refactor": "Refactorizar y optimizar el código actual",
        "@execute": "Ejecutar acción y herramientas agénticas",
    }
    if action == "@chat":
        subprocess.run(["eww", "open", "--toggle", "agent_chat"])
    else:
        prompt = prompts.get(action, action.lstrip("@"))
        # Lista de args — sin shell, sin interpolación
        subprocess.Popen(["ghostty", "-e", "hermes", "--prompt", prompt] +
                         shlex.split(extra_args))

    print(json.dumps({"ok": True, "action": action,
                      "ipc_sent": socket_sent}))
```

Añadir `import shlex` arriba. Borrar TODOS los `shell=True`.

**Step 2: Test de regresión**

Create: `tests/test_no_shell_injection.py`

```python
import ast, pathlib

FORBIDDEN = {"shell": True}

def test_no_shell_true_in_scripts():
    for py in pathlib.Path("eww/scripts").glob("*.py"):
        tree = ast.parse(py.read_text())
        for node in ast.walk(tree):
            if isinstance(node, ast.keyword) and node.arg == "shell":
                assert getattr(node.value, "value", None) is not True, \
                    f"shell=True prohibido en {py}"
```

Run: `python3 -m pytest tests/test_no_shell_injection.py -v`
Expected: PASS.

**Step 3: Commit**

```bash
git add eww/scripts/hermes_orb_menu.py tests/test_no_shell_injection.py
git commit -m "security: eliminate shell=True command injection in orb menu dispatcher"
```

### Task 1.2: Sockets con XDG_RUNTIME_DIR (adiós UID 1000 fijo)

**Objective:** Telemetry socket respeta `$XDG_RUNTIME_DIR`.

**Files:**
- Modify: `crates/swal-node-daemon/src/main.rs:57` (y donde se use)
- Modify: `crates/swal-telemetry-rs/src/ipc.rs` (si repite la constante)

**Step 1: Helper portátil**

En `main.rs`:

```rust
fn runtime_dir() -> std::path::PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            std::path::PathBuf::from(format!("/run/user/{}", uid))
        })
}
// Uso:
let sock_path = runtime_dir().join("swal").join("telemetry.sock");
```

**Step 2: Test**

```rust
#[test]
fn telemetry_socket_uses_xdg_runtime_dir() {
    std::env::set_var("XDG_RUNTIME_DIR", "/tmp/test-xdg");
    let p = runtime_dir().join("swal").join("telemetry.sock");
    assert!(p.starts_with("/tmp/test-xdg"));
}
```

Run: `cargo test -p swal-node-daemon telemetry_socket -v` → PASS.

**Step 3: Commit**

```bash
git add crates/swal-node-daemon/src/main.rs
git commit -m "security(portability): derive IPC sockets from XDG_RUNTIME_DIR instead of fixed UID"
```

### Task 1.3: Control socket en runtime dir + permisos 0600

**Objective:** `/tmp/swal_desktop_ctl.sock` (world-writable dir) → `$XDG_RUNTIME_DIR/swal/ctl.sock` con perms usuario-only.

**Files:**
- Modify: `crates/swal-node-daemon/src/main.rs` (`DEFAULT_CTL_SOCKET`)
- Test: inline `#[cfg(test)] mod tests`

**Step 1:** Cambiar constante por función (misma técnica que 1.2). Tras `UnixListener::bind`, aplicar:

```rust
use std::os::unix::fs::PermissionsExt;
let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
```

**Step 2:** Test: bind temporal en tmpdir, assert mode & 0o077 == 0.

**Step 3:** Run: `cargo test -p swal-node-daemon ctl_socket -v` → PASS.

**Step 4: Commit**

```bash
git add crates/swal-node-daemon/src/main.rs
git commit -m "security: move control socket to XDG_RUNTIME_DIR with 0600 permissions"
```

---

## FASE 2 — Portabilidad total (repo usable por cualquiera)

### Task 2.1: Helper canónico de home en Rust

**Objective:** Una sola fuente de verdad para $HOME.

**Files:**
- Create: `crates/swal-node-daemon/src/paths.rs`

**Step 1: Implementación**

```rust
use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"))
        .join("swal")
}

pub fn eww_scripts_dir() -> PathBuf {
    config_dir().parent().unwrap().join("eww").join("scripts")
}
```

Exportar en `lib.rs`: `pub mod paths;`

**Step 2: Commit**

```bash
git add crates/swal-node-daemon/src/paths.rs crates/swal-node-daemon/src/lib.rs
git commit -m "feat(core): canonical portable path helpers (home/config/xdg)"
```

### Task 2.2: Erradicar /home/belal de src/ (32 → 0)

**Objective:** Cero rutas personales en código productivo.

**Files (grep verificado):**
- Modify: `crates/swal-node-daemon/src/main.rs` (6 sitios: usar `paths::eww_scripts_dir().join("toggle_dashboard.sh")` etc.)
- Modify: `crates/swal-files/src/cli.rs`, `gui.rs`, `config.rs`, `session.rs`, `preview.rs`, `omnibar.rs`
- Modify: `crates/swal-vision-rs/src/gesture_ipc.rs`:
  `pub const GESTURE_CONFIG_PATH` → `pub fn gesture_config_path() -> PathBuf { crate::paths::config_dir().join("gesture.json") }` (adaptar import al crate correcto).

**Step 1: Barrido mecánico**

```bash
# Encontrar todos
grep -rn "/home/belal" crates/*/src/
# Regla: fallback unwrap_or_else(|| PathBuf::from("/home/belal")) → dirs::home_dir().unwrap_or_default()
```

**Step 2: Gate anti-regresión**

Create: `tests/test_no_hardcoded_paths.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
HITS=$(grep -rn "/home/belal" crates/*/src/ || true)
if [ -n "$HITS" ]; then
  echo "❌ Paths hardcodeados detectados:"; echo "$HITS"; exit 1
fi
echo "✓ Sin paths hardcodeados"
```

Run: `chmod +x tests/test_no_hardcoded_paths.sh && ./tests/test_no_hardcoded_paths.sh` → PASS.

**Step 3: Commit**

```bash
git add -A
git commit -m "portability: remove all hardcoded /home/belal paths from source"
```

### Task 2.3: Tests portables

**Objective:** Los tests usan `dirs::home_dir()` o tmpdirs, nunca rutas absolutas de belal.

**Files:**
- Modify: `crates/swal-files/tests/{test_full_core_coverage,test_standalone_crossplatform_e2e,test_file_viewer_formats}.rs`

**Step 1:** Reemplazar `"/home/belal/proyectosSWAL/periferia/swal-desktop"` por `env!("CARGO_MANIFEST_DIR")` (raíz del crate durante tests). Reemplazar fallbacks `"/home/belal"` por `std::env::temp_dir()`.

**Step 2:** Extender el gate de 2.2 a `crates/*/tests/` cuando pase verde.

**Step 3: Commit**

```bash
git add crates/swal-files/tests/ tests/test_no_hardcoded_paths.sh
git commit -m "portability(tests): replace personal absolute paths with cargo/env-relative ones"
```

### Task 2.4: NixOS module parametrizado

**Objective:** `nixos/swal-node.nix` funciona para cualquier usuario vía opción configurable.

**Files:**
- Modify: `nixos/swal-node.nix`

**Step 1: Convertir a module con opciones**

```nix
{ config, pkgs, lib, ... }:
with lib;
let
  cfg = config.services.swal-node;
in {
  options.services.swal-node = {
    enable = mkEnableOption "SWAL Autonomous Node";
    user = mkOption { type = types.str; default = "belal"; };
    workspaceDir = mkOption { type = types.str; example = "/home/user/proyectosSWAL"; };
  };

  config = mkIf cfg.enable {
    users.users.${cfg.user} = {};  # ya existente en host
    systemd.user.services.xavier-core.serviceConfig.ExecStart =
      "${pkgs.bash}/bin/bash -c 'exec ${cfg.xavierBin}/bin/xavier http 8006'";
    # ... Environment con ${cfg.workspaceDir} en lugar de /home/belal
  };
}
```

**Step 2:** Actualizar `flake.nix` para pasar `user` desde `config.users.users` primario.

**Step 3:** Validar sintaxis: `nix-instantiate --parse nixos/swal-node.nix` → sin error.

**Step 4: Commit**

```bash
git add nixos/swal-node.nix flake.nix
git commit -m "portability(nixos): parameterized swal-node module (user/workspace as options)"
```

### Task 2.5: install.sh idempotente y sin suposiciones de usuario

**Objective:** Instala en cualquier máquina NixOS/no-NixOS sin romper nada ajeno.

**Files:**
- Modify: `scripts/install.sh`

**Step 1: Cambios**
- `INSTALL_DIR="$HOME/proyectosSWAL/periferia/swal-desktop"` → clonar si no existe (`git clone "$REPO_URL" "$INSTALL_DIR"`).
- No tocar `/etc/nixos/*` salvo flag `--nixos` explícito (opt-in, no opt-out).
- Nunca ejecutar `sudo nixos-rebuild` automáticamente: imprimir instrucciones.
- `set -euo pipefail` ya está ✓.

**Step 2: Smoke test local**

Run: `bash -n scripts/install.sh && INSTALL_DIR=/tmp/swal-test bash scripts/install.sh --dry-run` (añadir soporte `--dry-run` que solo imprime acciones).

**Step 3: Commit**

```bash
git add scripts/install.sh
git commit -m "feat(installer): opt-in nixos integration, dry-run mode, clone-if-missing"
```

---

## FASE 3 — Zero-EWW real (cerrar la coexistencia)

### Task 3.1: Toggle dashboard/orb nativo en daemon

**Objective:** El daemon ya no depende de scripts .sh externos.

**Files:**
- Modify: `crates/swal-node-daemon/src/main.rs` (los 6 `Command::new("...toggle_*.sh")`)
- Create: `crates/swal-node-daemon/src/shell_actions.rs`

**Step 1: Acción nativa**

El supervisor YA tiene broadcast_event(ShellEvent::Command { surface: TelemetryBar, command: "toggle_dashboard" }). El fallback a .sh es lo que sobra:

```rust
// shell_actions.rs
pub async fn toggle_surface(supervisor: &NativeShellSupervisor, kind: NativeSurfaceKind, cmd: &str) {
    let _ = supervisor.broadcast_event(ShellEvent::Command {
        surface: kind,
        command: cmd.to_string(),
    });
    // SIN fallback a eww/scripts — la superficie nativa responde por IPC
}
```

Eliminar los 6 bloques `Command::new(...eww...)`. Mantener `spawn swal-files` pero vía `paths::` + `which`-style lookup (`~/.local/bin/swal-files` → buscar en PATH).

**Step 2: Test E2E mínimo**

Test que emite ShellEvent::Command y verifica que el router IPC recibe el evento (ya existe infra en `native_shell.rs`; añadir assertion).

**Step 3: Commit**

```bash
git add crates/swal-node-daemon/
git commit -m "feat(zero-eww): native toggle actions, drop external shell script fallbacks"
```

### Task 3.2: Migrar swal_settings.py a CLI Rust

**Objective:** El backend de settings vive en `settings_cli.rs` (ya existe, tiene `fn main` muerto — revivirlo).

**Files:**
- Modify: `crates/swal-node-daemon/src/settings_cli.rs:311` (exponer bin)
- Modify: `crates/swal-node-daemon/Cargo.toml` (`[[bin]] name = "swal-settings"`)

**Step 1:** Renombrar `fn main()` interno → `pub fn run(args: &[String]) -> i32` y crear bin wrapper fino.
Comandos a cubrir (paridad con el .py): `status | switch_theme <t> | restart_xavier | doctor [--fix] | rebuild-nix | set-profile <p>`.

**Step 2:** `subprocess.run(["systemctl"...])` equivalentes con `Command::new("systemctl").args([...])` — sin shell.

**Step 3: Test**

```rust
#[test]
fn settings_cli_status_parses_without_panics() {
    // status offline-safe: Xavier caído debe responder JSON, no panic
    let code = run(&["status".into()]);
    assert_eq!(code, 0);
}
```

**Step 4: Commit**

```bash
git add crates/swal-node-daemon/
git commit -m "feat(settings): revive settings_cli as swal-settings binary replacing python backend"
```

### Task 3.3: hermes_orb actions nativas (eliminar el .py)

**Objective:** El menú del orbe despacha por IPC Unix directamente desde Rust (el socket ya existe en ambient-orb/socket.rs).

**Files:**
- Modify: `crates/swal-ambient-orb/src/socket.rs` (añadir `send_action(action: &str)`)
- Modify: `crates/swal-a2ui-engine/src/hermes_streamer.rs` (referencias eww)

**Step 1:** Función cliente:

```rust
pub fn send_action(sock_path: &Path, action: &str, extra: &str) -> Result<(), std::io::Error> {
    let payload = serde_json::json!({
        "event": "action_triggered", "action": action,
        "extra_args": extra, "timestamp": now_secs()
    });
    UnixStream::connect(sock_path)?.write_all(payload.to_string().as_bytes())
}
```

CLI bin pequeño `swal-orb-action @summarize|@refactor|@execute|@chat`.

**Step 2:** Marcar `eww/scripts/hermes_orb_menu.py` DEPRECATED en header y borrar en 3.5.

**Step 3: Commit**

```bash
git add crates/swal-ambient-orb/ crates/swal-a2ui-engine/
git commit -m "feat(orb): native IPC action client replaces python dispatcher"
```

### Task 3.4: Paridad visual antes de apagar Eww

**Objective:** Confirmar que las superficies nativas cubren TODO lo que Eww renderiza hoy.

**Files:**
- Create: `docs/EWW_PARITY_CHECKLIST.md`

**Step 1: Inventario**

Listar widgets de `eww/eww.yuck` (76KB — barre defwindow/defwidget): dashboard, telemetry bar, orb HUD, agent_chat, files-fluent, hermes_orb, ram_panel, ai_status, sys_info.

**Step 2:** Por cada uno marcar: superficie nativa equivalente (render-pipeline/a2ui) — SÍ/NO/PARCIAL. Los NO/parciales generan tasks hijas (una por widget) ANTES de seguir a 3.5.

**Step 3:** Ejecutar `scripts/swal-doctor` y `scripts/swal-visual-test`; pegar salida en el checklist.

**Step 4: Commit**

```bash
git add docs/EWW_PARITY_CHECKLIST.md
git commit -m "docs: eww→native parity checklist with per-widget status"
```

### Task 3.5: Apagado limpio de Eww

**Objective:** Repo sin directorio eww/ activo; historial preserva el legado.

**PRECONDICIÓN:** Checklist 3.4 al 100% SÍ.

**Step 1:**
```bash
git rm -r eww/
# conservar en archivo/ si hay valor histórico:
mkdir -p ../archivo/swal-desktop-eww-legacy
cp -r eww/ ../archivo/swal-desktop-eww-legacy/  # fuera del repo
```
Actualizar `hypr/` bindings: SUPER+Escape/SUPER+E/SUPER+Q → `swal-node-daemon toggle-*` (ya existen handlers).
Actualizar `flake.nix` y `nixos/configuration.nix` quitando paquetes eww.
Actualizar README (sección Coexistence → Native Only) y `docs/RUST_MIGRATION_ARCHITECTURE.md` estado final.

**Step 2: Verificar**

Run: `./tests/test_no_hardcoded_paths.sh && cargo test --workspace 2>&1 | grep "test result"`
Y grep global: `grep -rn "eww" crates/ hypr/ nixos/ flake.nix` → solo comentarios históricos permitidos.

**Step 3: Commit**

```bash
git add -A
git commit -m "feat!: zero-eww milestone — native Rust shell is the only shell"
```

---

## FASE 4 — Higiene y calidad

### Task 4.1: Cero warnings de compilación

**Objective:** `cargo build --workspace` sin warnings.

**Files:**
- Modify: `crates/swal-a2ui-engine/src/standalone_window.rs:82` (usar `_max_label` o implementarlo)
- Modify: `crates/swal-ambient-orb/src/main.rs:5` (quitar `OrbState` del use)
- Modify: `crates/swal-a2ui-engine/src/main.rs:5` (quitar `ThemePalette`)
- Modify: `crates/swal-node-daemon/src/tests/test_wave7_session_release_e2e.rs:4-10` (limpiar imports)
- Modify: `crates/swal-files/src/cli.rs` (borrar `EWWSOCK` muerto y `cleanup_orphan_windows` si 3.1 los hizo obsoletos)

**Step 1:** `cargo fix --workspace --allow-dirty --all-targets` + revisión manual de lo que no autofixa.

**Step 2: Gate duro en Cargo.toml raíz**

```toml
[workspace.lints.rust]
warnings = "deny"
```

Run: `cargo clippy --workspace --all-targets 2>&1 | grep -c warning` → 0.

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: zero warnings, deny-by-default lints"
```

### Task 4.2: Reducir unwrap() de alto riesgo en swal-files

**Objective:** Sin panics en rutas de IO de usuario (archive.rs=25, cloud_sync.rs=19, tui.rs=7...).

**Step 1:** Política: `unwrap()` SOLO en tests y en `lock().unwrap()` de Mutex. En src/: `anyhow::Context` + `?` o `expect("INVARIANTE: ...")` documentado.

**Step 2:** Priorizar archivos top-5 del conteo. No requiere llegar a 0 absoluto: gate = `cargo clippy` con lint custom:
`#![deny(clippy::unwrap_used)]` en los 5 archivos top (gradual).

**Step 3: Commit**

```bash
git add crates/swal-files/
git commit -m "refactor(swal-files): replace high-risk unwraps with propagated errors (top-5 files)"
```

---

## FASE 5 — CI y publicación segura

### Task 5.1: GitHub Actions pipeline

**Objective:** Todo PR/push corre tests + seguridad.

**Files:**
- Create: `.github/workflows/ci.yml`

```yaml
name: ci
on: [push, pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - uses: Swatinem/rust-cache@v2
      - run: sudo apt-get update && sudo apt-get install -y libwayland-dev libxkbcommon-dev
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
      - run: cargo install cargo-audit --locked || true
      - run: ~/.cargo/bin/cargo-audit audit || true   # warn-first, endurecer luego
  hygiene:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: gacts/gitleaks@v1          # secretos
      - run: ./tests/test_no_hardcoded_paths.sh
  python-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pip install pytest && python3 -m pytest tests/ -v
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: full pipeline — clippy deny-warnings, tests, gitleaks, path-hygiene gate"
```

### Task 5.2: Saneo pre-publicación del contenido

**Objective:** Repo público sin datos privados.

**Step 1: Auditoría de historia**

```bash
gitleaks detect --source . --redact -v   # historia completa
```
Si halla secretos históricos → `git filter-repo` + force-push coordinado + rotar credenciales afectadas.

**Step 2: Revisar docs con info personal**

`grep -rn "belal\|iberi22@gmail\|Belalcazar" README.md docs/ scripts/ themes/ schemas/`
Decidir: autoría en LICENSE/README OK; emails personales en configs NO.

**Step 3: .gitignore reforzado**

```
target/
*.sock
.env*
!.env.example
__pycache__/
*.log
/tmp/
```
Verificar que `debug-rebuild.sh` y logs de `/tmp/swal-debug.log` no estén trackeados.

**Step 4: Commit**

```bash
git add .gitignore
git commit -m "chore: hardened gitignore, privacy pass over docs"
```

### Task 5.3: Push y release v1.3.0

**Objective:** Publicar el hito.

**Step 1:**
```bash
cargo test --workspace          # verde
./tests/test_no_hardcoded_paths.sh  # verde
python3 -m pytest tests/ -v     # verde
git push origin main
```

**Step 2:** Tag:
```bash
git tag -a v1.3.0-zero-eww -m "Zero-EWW: pure Rust native shell, portable, security-hardened"
git push origin v1.3.0-zero-eww
```

**Step 3:** GitHub Release con changelog de CHANGELOG.md + nota de seguridad (qué se corrigió respecto al patrón Omarchy: sin shell=True, sockets 0600 en XDG_RUNTIME_DIR, CI con gitleaks).

---

## Dependencias entre fases

```
Fase 0 (compila) ──► Fase 1 (seguro) ──► Fase 2 (portable) ──► Fase 3 (zero-eww) ──► Fase 4 (higiene) ──► Fase 5 (publicar)
                          │                                        │
                          └── 3.4 puede empezar en paralelo        └── 3.5 REQUIERE checklist 3.4 = 100%
```

## Definition of Done (verificable)

1. `cargo test --workspace` → 0 failed, 0 warnings
2. `grep -rn "/home/belal" crates/ nixos/ scripts/install.sh` → 0 resultados
3. `grep -rn "shell=True" eww/ scripts/` → 0 (o eww/ eliminado)
4. Sockets en `$XDG_RUNTIME_DIR` con permiso 0600 (test unitario verde)
5. `ls eww/` → No such file (post 3.5) + parity checklist 100%
6. `.github/workflows/ci.yml` verde en main
7. `gitleaks detect` → 0 findings
8. Push + tag v1.3.0 publicados

## Execution Log

| Fecha | Fase | Acción | Estado |
|---|---|---|---|
| 2026-08-25 | Audit | Diagnóstico completo (compilación rota E0063+linking, 32 paths, shell=True×4, sin CI) | ✅ |
| 2026-08-25 | Fase 0 | Task 0.1+0.2: E0063 ×3 corregidos | ✅ |
| 2026-08-25 | Fase 0 | Task 0.3: linking gcc = entorno NixOS; fix via RUSTFLAGS `-L/-B /nix/store/…-gcc-prefix/lib`. NOTA: persistir en `.cargo/config.toml` o nix devshell para CI | ✅ |
| 2026-08-25 | Fase 0 | Bonus: flaky test `test_mode_auto_detection_standalone_window` (carrera SWAL_HEADLESS entre tests paralelos) — serializado con mutex compartido. 76/76 swal-files lib verde ×2 pasadas; workspace completo exit 0 | ✅ |
| — | Fase 1 | Seguridad P0 (shell=True, sockets XDG) | ⏳ |
| 2026-08-25 | Fase 1 | Task 1.1: shell=True ×4 eliminados en hermes_orb_menu.py + test AST anti-regresión (2/2 ok) | ✅ |
| 2026-08-25 | Fase 1 | Task 1.2+1.3: ctl+telemetry sockets → \$XDG_RUNTIME_DIR/swal/, dir 0700, socket 0600; libc dep añadida; clientes a2ui/orb con fallback legacy; 18/18 tests bin verde | ✅ |
| — | Fase 2 | Portabilidad (32 paths /home/belal, nix module, installer) | ⏳ |
| — | Fase 3 | Zero-EWW (toggles nativos, settings CLI, parity checklist) | ⏳ |
| — | Fase 4 | Higiene (warnings → deny, unwraps) | ⏳ |
| — | Fase 5 | Publicar (CI, gitleaks, push + tag v1.3.0) | ⏳ |
