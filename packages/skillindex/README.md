# SkillIndex

Detecta e instala las mejores skills de agentes IA para tu proyecto. Un comando, cero configuración.

```bash
cargo install skillindex
skillindex --help
```

`skillindex` escanea tu proyecto, detecta las tecnologías que usas e instala skills curadas que hacen que Cursor, Claude Code y otros asistentes realmente entiendan tu stack.

## Instalación

**Con Cargo (Rust):**

```bash
cargo install skillindex
```

**Con npm (Node.js):**

```bash
npx skillindex
npm i -g skillindex
```

## Uso

```bash
# En la raíz de tu proyecto
skillindex

# Omitir confirmación
skillindex -y

# Vista previa sin instalar
skillindex --dry-run

# Elegir IDEs específicos
skillindex -a kiro opencode
skillindex -a claude-code --dry-run
```

## Opciones

| Flag              | Descripción                                                                              |
| ----------------- | ---------------------------------------------------------------------------------------- |
| `-y`, `--yes`     | Omite la confirmación (instala en todos los agentes detectados)                          |
| `--dry-run`       | Muestra las skills sin instalar                                                          |
| `--clear-cache`   | Limpia la caché de skills descargadas                                                    |
| `-a`, `--agent`   | Instala solo para IDEs específicos (ej. `cursor`, `claude-code`, `opencode`, `kiro-cli`) |
| `-v`, `--verbose` | Muestra traza y detalles de errores                                                      |
| `-h`, `--help`    | Muestra la ayuda                                                                         |

## Agentes soportados

Detecta automáticamente los IDEs instalados en tu `$HOME` y delega la elección a vos:

- **Claude Code** (`.claude`), **Cursor** (`.cursor`), **Opencode** (`.opencode`), **Kiro** (`.kiro`), **Cline** (`.cline`), **Junie** (`.junie`), **CodeBuddy** (`.codebuddy`), **Continue** (`.continue`)
- Si detecta más de un agente, te muestra un selector para elegir dónde instalar (por defecto todos seleccionados, podés destildar `.kiro` si solo usás opencode/cursor).
- Con `-a` o `-y` se respeta tu elección sin preguntar; `--dry-run` muestra los agentes elegidos sin instalar.

## Tecnologías soportadas

Más de 50 tecnologías: React, Next.js, Vue, Svelte, Astro, Tailwind, TypeScript, Go, Rust, Python, Supabase, etc. Ver [README completo](https://github.com/Gabox301/SkillIndex#tecnologías-soportadas) para la lista completa.

## Licencia

MIT — Creado por [Gabriel Ortega](https://github.com/Gabox301)
