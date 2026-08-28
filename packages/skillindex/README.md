# SkillIndex

Detecta e instala automáticamente las mejores skills de agentes IA para tu proyecto. Un comando, cero configuración.

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
```

## Opciones

| Flag | Descripción |
|------|-------------|
| `-y`, `--yes` | Omite la confirmación |
| `--dry-run` | Muestra las skills sin instalar |
| `-v`, `--verbose` | Muestra traza y detalles de errores |
| `-h`, `--help` | Muestra la ayuda |

## Tecnologías soportadas

Más de 50 tecnologías: React, Next.js, Vue, Svelte, Astro, Tailwind, TypeScript, Go, Rust, Python, Supabase, etc. Ver [README completo](https://github.com/Gabox301/SkillIndex#tecnologías-soportadas) para la lista completa.

## Licencia

MIT — Creado por [Gabriel Ortega](https://github.com/Gabox301)
