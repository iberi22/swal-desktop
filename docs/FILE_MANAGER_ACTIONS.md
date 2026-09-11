# SWAL Files — Documentación de Acciones, Menú Contextual y Portapapeles (Ola FM)

El Administrador de Archivos SWAL Files (`swal-files`) proporciona un conjunto completo de comandos CLI y acciones de menú contextual diseñadas para la gestión eficiente de archivos y directorios en entornos Wayland/NixOS.

---

## 1. Tabla de Acciones del File Manager

| Acción | Comando CLI | Atajo Sugerido | Notas & Comportamiento |
| :--- | :--- | :--- | :--- |
| **Menú Contextual** | `swal-files menu-json <path>` | `Right-Click` / `Menu` | Devuelve JSON con >= 13 acciones contextuales y estado de `paste_enabled`. |
| **Copiar al Portapapeles** | `swal-files clip-copy <path...>` | `Ctrl+C` | Registra archivos en el manifiesto temporal del portapapeles en modo copia. |
| **Cortar al Portapapeles** | `swal-files clip-cut <path...>` | `Ctrl+X` | Registra archivos en el manifiesto temporal del portapapeles en modo mover. |
| **Pegar Portapapeles** | `swal-files clip-paste <dir>` | `Ctrl+V` | Pega/mueve archivos. En caso de colisión de nombre, no sobrescribe (añade sufijo `_copy`). |
| **Renombrar Item** | `swal-files rename-item <path> <new_name>` | `F2` | Valida que el nuevo nombre no contenga separadores `/`. Falla sin alterar el disco. |
| **Eliminar Item** | `swal-files delete-item <path> [--confirm=BORRAR]` | `Delete` / `Shift+Delete` | Requiere confirmación explícita para evitar borrados accidentales sin efectos no deseados. |
| **Ver Propiedades** | `swal-files properties-json <path>` | `Alt+Enter` | Reporta metadatos JSON completos incluyendo `mode_octal` (permisos POSIX) y `size_bytes`. |
| **Navegar a Directorio** | `swal-files nav <dir>` | `Enter` | Cambia el directorio activo de la pestaña actual. |
| **Seleccionar Item** | `swal-files select-item <path>` | `Left-Click` | Marca un archivo o directorio como seleccionado en la sesión activa. |
| **Abrir Item / Editor** | `swal-files open-item <path>` | `Double-Click` | Abre carpetas en la pestaña activa o archivos en el visor/editor flotante. |
| **Alternar Vista** | `swal-files toggle-view` | `Ctrl+1` / `Ctrl+2` | Alterna entre vista de detalles (`details`) y vista en cuadrícula (`grid`). |
| **Alternar Ocultos** | `swal-files toggle-hidden` | `Ctrl+H` | Muestra u oculta archivos y directorios dotfile (`.filename`). |
| **Ver Texto / Copiar Línea** | `swal-files view-text <path>` / `copy-line` | `Ctrl+Shift+C` | **Limitación EWW**: Los labels de EWW no son seleccionables con ratón, por lo que se proveen estos comandos dedicados para lectura y copiado de texto. |

---

## 2. Integración y Cableado con UI (EWW Wiring)

El flujo del menú contextual e interacción entre EWW y `swal-files` sigue la siguiente arquitectura:

1. **Invocación del Menú Contextual (`menu-json`)**:
   - Al hacer clic derecho sobre un archivo o directorio en la interfaz de EWW, EWW invoca `swal-files menu-json <path>`.
   - La respuesta JSON contiene la lista de acciones habilitadas (abrir, renombrar, copiar, cortar, pegar, propiedades, etc.) y banderas de estado como `paste_enabled`.

2. **Renderizado del Menú (`files_ctx_menu`)**:
   - El resultado de `menu-json` se asigna a la variable EWW `files_ctx_menu`.
   - El widget popup de menú contextual en `eww/eww.yuck` itera recursivamente sobre el array `actions` renderizando botones dinámicos con sus respectivos iconos SCSS (`eww/files-fluent.scss`).

3. **Ejecución de Acciones (`menu-run`)**:
   - Al seleccionar una opción del menú contextual, el evento de clic llama al script o helper `menu-run <action> <path>`.
   - `menu-run` canaliza la acción elegida hacia el comando CLI correspondiente de `swal-files` (e.g. `clip-copy`, `rename-item`, `delete-item`).
   - Una vez finalizada la operación en disco, `swal-files` emite un evento de actualización que refresca la vista JSON de EWW (`view-json`).

---

## 3. Limitaciones Conocidas & Soluciones

- **Selección de Texto en Labels de EWW**:
  Debido a restricciones del motor de widgets GTK3/EWW 0.6, las etiquetas de texto (`label`) no admiten selección directa por cursor ni copiar al portapapeles nativo. Para mitigar esta restricción, SWAL Files provee los comandos subyacentes `view-text` y `copy-line` para inspeccionar y copiar líneas de texto específicas desde el visor flotante.
