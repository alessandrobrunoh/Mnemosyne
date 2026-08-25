

# Mnemosyne (MNP)

**Arqueología Semántica de Código y Memoria de Alto Rendimiento para tu Base de Código.**

Mnemosyne es una plataforma de alto rendimiento, local-first, diseñada para proporcionar una memoria eterna para tu flujo de trabajo de desarrollo. Aprovechando Tree-sitter para la comprensión semántica y una arquitectura de almacenamiento direccionable por contenido (CAS), Mnemosyne rastrea la evolución de tu código no solo como diferencias de texto, sino como mutaciones lógicas de símbolos, funciones y estructuras.

[![CI](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/ci.yml/badge.svg)](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/ci.yml)
[![Build](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/build.yml/badge.svg)](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

---

## Pilares Fundamentales

### 1. Identidad Semántica sobre Diferencias de Texto
El control de versiones tradicional rastrea líneas. Mnemosyne rastrea la **lógica**. Utilizando ASTs de Tree-sitter y algoritmos de `structural_hash`, mantiene la continuidad a través de cambios de nombre, refactorizaciones y movimientos. Comprende cuándo una función se ha movido o cuándo una estructura ha evolucionado, proporcionando un verdadero registro "arqueológico" de tus símbolos.

### 2. Ingeniería de Alto Rendimiento
Construido en Rust con un enfoque en operaciones zero-copy y concurrencia granular:
*   **Manejo de Datos Zero-Copy**: Utiliza `bytes::Bytes` y `mmap` para minimizar la sobrecarga de la CPU.
*   **Concurrencia Lock-Free**: Utiliza colecciones concurrentes (`DashMap`) y operaciones atómicas para garantizar que el demonio nunca bloquee tu flujo de trabajo.
*   **Arquitectura de Almacenamiento**: Impulsado por `redb` como un índice B-tree de alto rendimiento y una capa CAS personalizada para una deduplicación eficiente.

### 3. Integración con el Protocolo de Contexto de Modelo (MCP)
Mnemosyne no es solo para humanos. Actúa como una potente capa de contexto para LLMs y agentes de IA a través del **Protocolo Mnemosyne (MNP)**. Permite que los agentes:
*   Recuperen deltas semánticos en lugar de archivos sin procesar (ahorrando tokens).
*   Accedan a la evolución histórica de símbolos específicos.
*   Comprendan el "por qué" detrás de los cambios arquitectónicos mediante metadatos indexados.

---

## Arquitectura del Espacio de Trabajo

El proyecto está estructurado como un espacio de trabajo modular para máxima reutilización:

| Componente | Descripción |
|-----------|-------------|
| `mnem-daemon` | El motor en segundo plano que gestiona la observación de archivos, el análisis AST y el almacenamiento. |
| `mnem-cli` | Una CLI pulida para la gestión de proyectos y el acceso rápido al historial. |
| `mnem-tui` | Una interfaz de terminal rica en funciones para la exploración visual del historial. |
| `mnem-core` | Lógica compartida: IPC, protocolo MNP, almacenamiento CAS y esquemas de base de datos. |
| `mnem-mcp` | Servidor del Protocolo de Contexto de Modelo para integrarse con LLMs (Claude, etc.). |
| `mnem-lsp` | Puente del Protocolo del Servidor de Lenguaje para funciones nativas del IDE. |
| `mnem-zed` | Extensión nativa para el editor Zed. |

---

## Primeros Pasos

### Instalación

#### Instalación en una Línea (Recomendada)
**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.ps1 | iex
```

#### Desde el Código Fuente
```bash
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne
cargo build --release
```

### Inicio Rápido
```bash
# Iniciar el demonio en segundo plano
mnem daemon start

# Iniciar el seguimiento en tu proyecto actual
mnem track

# Abrir la TUI interactiva para explorar el historial
mnem ui

# Buscar el historial de un símbolo específico
mnem search --symbol "process_data"
```

---

## Configuración

Mnemosyne respeta los límites de tu proyecto a través de archivos `.mnemignore` y un `config.toml` global.

**Configuración Global (`~/.mnemosyne/config.toml`):**
```toml
[storage]
retention_days = 30
compression_enabled = true
max_file_size_mb = 10

[editor]
ide = "Zed"
```

---

## Licencia

Este proyecto está licenciado bajo la **Licencia Apache 2.0**. Consulta el archivo [LICENSE](LICENSE) para más detalles.
