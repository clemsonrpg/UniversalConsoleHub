# Arquitetura do Sistema de Temas — Universal Console Hub 3.0

O sistema de temas do **Universal Console Hub 3.0** foi projetado com inspiração na flexibilidade do Playnite, permitindo alterar a experiência visual completa do aplicativo — **estrutura de navegação (shell), paleta, tipografia, geometria de superfícies, estilo de foco do controle, transições e arte de fundo** — sem comprometer a segurança do sistema.

---

## 1. Modelo Declarativo e Seguro (Zero Execução de Código)

Diferentemente de sistemas baseados em scripts ou CSS arbitrário, pacotes de tema no Universal Console Hub são estritamente **declarativos**:

- **Zero JavaScript / HTML / CSS arbitrário:** nenhum arquivo executável, `<script>`, `url(...)` externo, `expression(...)` ou `@import` é permitido.
- **Validação no Backend Rust (`src-tauri/src/domain/theme.rs` & `application/theme_service.rs`):**
  - Valida `id` em `kebab-case` (rejeita IDs reservados dos 4 temas nativos).
  - Valida paleta de cores (apenas hex `#RGB`/`#RRGGBB`/`#RRGGBBAA` ou `rgba(...)`/`rgb(...)`).
  - Valida fontes contra lista segura (bloqueia `;`, `{`, `}`, `url(`, `<`, `>`).
  - Valida layouts (`sidebar`, `topbar_blades`, `minimal_dock`, `xmb_crossbar`), estilos de foco (`ring`, `glow_bar`, `bracket`, `xmb_wave`) e transições (`fade`, `slide`, `scale`, `none`) contra enums fechados.
  - Valida todas as imagens locais (`previewImage`, `backgroundImage`, `assets`) contra path traversal (`..`, caminhos absolutos), tamanho máximo (`8 MB`) e **magic bytes reais** (PNG, JPEG ou WebP).
  - Instala pacotes validados exclusivamente em `%LOCALAPPDATA%\com.universalconsolehub.desktop\themes\<theme-id>\`, servido em modo somente leitura via escopo explícito `$APPLOCALDATA/themes/**` do `assetProtocol` no Tauri 2.

---

## 2. Formato Documentado do Pacote (`theme.json`)

Um pacote local pode ser importado no **Gerenciador de Temas** como uma pasta contendo `theme.json` (+ subpasta `assets/` opcional) ou um arquivo `.json` isolado.

```json
{
  "id": "obsidian-emerald",
  "name": "Obsidian Emerald",
  "author": "Comunidade Hub",
  "version": "1.0.0",
  "description": "Tema escuro de alto contraste com lâminas superiores esmeralda.",
  "hubVersionRange": ">=0.1.0",
  "previewImage": "assets/preview.png",
  "backgroundImage": "assets/backdrop.webp",
  "fonts": {
    "display": "'Segoe UI Variable Display', 'Segoe UI', sans-serif",
    "body": "'Segoe UI Variable Text', 'Segoe UI', sans-serif",
    "mono": "'Cascadia Code', 'Consolas', monospace"
  },
  "assets": {},
  "palette": {
    "canvas": "#0b1114",
    "rail": "#10191e",
    "surface": "#152127",
    "surfaceRaised": "#1d2d35",
    "input": "#0d1519",
    "borderSubtle": "#263a44",
    "borderStrong": "#3b5867",
    "textPrimary": "#f1f7f9",
    "textSecondary": "#a4bcc7",
    "textMuted": "#6e8794",
    "accentPrimary": "#10b981",
    "accentHover": "#34d399",
    "accentMuted": "rgba(16, 185, 129, 0.18)",
    "accentSecondary": "#38bdf8",
    "focusRing": "#10b981"
  },
  "layout": {
    "shell": "topbar_blades",
    "libraryDefaultView": "grid",
    "cardAspect": "poster",
    "density": "comfortable"
  },
  "components": {
    "focusStyle": "glow_bar",
    "surfaceStyle": "beveled",
    "backdropBlur": false,
    "showHeroArt": true,
    "showClock": true,
    "radiusSm": 6,
    "radiusMd": 10,
    "radiusLg": 16
  },
  "motion": {
    "pageTransition": "slide",
    "durationMs": 160,
    "respectReducedMotion": true
  },
  "customizableSettings": [
    {
      "key": "accentPrimary",
      "label": "Cor de destaque principal",
      "kind": "color",
      "defaultValue": "#10b981"
    },
    {
      "key": "coverScale",
      "label": "Escala das capas",
      "kind": "select",
      "defaultValue": "normal",
      "options": [
        { "value": "compact", "label": "Compacta" },
        { "value": "normal", "label": "Equilibrada" },
        { "value": "large", "label": "Grande (TV)" }
      ]
    }
  ]
}
```

---

## 3. Os 4 Temas Nativos Inclusos

Cada tema integrado possui **identidade visual, estrutura de navegação e foco próprios**:

| ID | Nome | Layout do Shell | Foco do Controle | Superfícies & Atmosfera |
|---|---|---|---|---|
| `universal-modern` | **Universal Modern** *(Padrão)* | `sidebar` (barra lateral fixa no Desktop; HUD superior no Fullscreen TV) | `ring` (anel calibrado aço-azul) | Grafite profundo fosco (`matte`), capas em destaque e arte de fundo controlada |
| `console-classic` | **Console Classic** | `topbar_blades` (lâminas horizontais de 7ª geração alternáveis por `LB`/`RB`) | `glow_bar` (barra tátil esmeralda) | Grafite metálico chanfrado (`beveled`), blocos largos (`wide`) e transição `slide` |
| `minimal` | **Minimal** | `minimal_dock` (dock superior limpo e compacto) | `bracket` (moldura arquitetural branca de alto contraste) | Superfícies planas (`flat`), contraste automático em botões claros, alta densidade |
| `xmb-inspired` | **XMB Inspired** | `xmb_crossbar` (barra cruzada horizontal de categorias + fluxo vertical) | `xmb_wave` (halo luminoso) | Horizonte procedural em CSS puro (`Safira`/`Obsidiana`/`Esmeralda`/`Rubi`) — **zero ativos proprietários da Sony** |

---

## 4. Persistência, Prévia e Personalização

- **Prévia ao vivo sem sobrescrever preferência:** `ThemeProvider` aplica `previewTheme` ao `document.documentElement` imediatamente com faixa de confirmação (**Aplicar** / **Cancelar**).
- **Persistência no SQLite (`application_settings`):**
  - `theme.active_id` — ID do tema ativo.
  - `theme.disabled_ids` — lista JSON de temas desativados (o padrão `universal-modern` nunca pode ser desativado).
  - `theme.overrides.<theme-id>` — mapa JSON de ajustes personalizáveis salvos por tema (`accentPrimary`, `coverScale`, `heroBackdropOpacity`, `xmbHorizon`, `animatedWave`, `showClock`).
- **Aplicação instantânea:** nenhuma troca de tema ou personalização exige reiniciar o aplicativo.
