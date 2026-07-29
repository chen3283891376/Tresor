# Tresor

Tauri v2 + React 19 + TypeScript encrypted password manager.

## Quick start

```bash
bun install
bun run tauri dev     # full Tauri desktop app (frontend + Rust backend)
bun run dev           # Vite frontend only (port 1420)
bun run build         # tsc + vite build (frontend only)
bun run tauri build   # production desktop build
bun run format        # prettier --write .
```

## Architecture

| Layer | Dir | Tech |
|-------|-----|------|
| Frontend | `src/` | React 19, TypeScript, Vite 7, Tailwind v4 |
| UI kit | `src/components/ui/` | shadcn/ui (base-nova style), lucide-react icons |
| State | `src/store/` | zustand |
| Backend | `src-tauri/src/` | Rust, Tauri v2 |
| Crypto | `src-tauri/src/` | argon2 + HKDF + AES-256-GCM |

Entrypoint: `src/main.tsx` → `App.tsx` (routes between LoginPage or VaultUnlockedView based on `isUnlocked` state).

## Key conventions

- **`@/` path alias** → `src/` (tsconfig paths + vite resolve alias)
- **Path imports** must include `.tsx`/`.ts` extension (e.g. `@/components/ui/sonner.tsx`)
- **Tailwind v4** — uses `@tailwindcss/vite` plugin, `@import 'tailwindcss'` in CSS (not postcss config)
- **Prettier**: 4-space indent, single quotes, avoid parens on arrow fns, 120 print width
- **TypeScript**: strict mode, `noUnusedLocals`/`noUnusedParameters` on, no unused imports allowed
- **License file** → `.key` file (picked via file dialog), combined with user password via argon2 + HKDF
- **Vault file format** on disk: `[16B salt][12B nonce][16B tag][ciphertext]`
- **Per-entry encryption** — each entry has its own AES-256 sub-key derived via HKDF(master_key, entry_id)

## Testing

No test framework is configured. No test suites exist.

## UI component patterns

- shadcn/ui components live in `src/components/ui/`, manually managed (no `shadcn add` needed unless adding a new one)
- `cn()` helper from `src/lib/utils.ts` merges Tailwind classes (clsx + tailwind-merge)
- Dark mode via `next-themes` ThemeProvider, CSS variables in `src/index.css` with `.dark` variant
- Sidebar component in `src/components/ui/sidebar.tsx` (shadcn sidebar pattern)

## Rust backend notes

- Single lib/bin crate: `src-tauri/src/lib.rs` (tauri `run()` fn), `src-tauri/src/main.rs` (calls `tresor_lib::run()`)
- All sensitive buffers use `zeroize` + memory locking (VirtualLock on Windows, mlock on Unix)
- `PR_SET_DUMPABLE = 0` set on Unix to prevent core dumps
- Ctrl+Alt+V global shortcut triggers simulated password paste via `enigo`
- Password leak detection uses HIBP k-anonymity API (SHA-1 prefix, 4 concurrent requests max)
