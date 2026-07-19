# Codex Usage Overlay

Drag it anywhere, adjust its appearance, then pin it to make it click-through. Use the tray menu or `Ctrl+Alt+Shift+U` whenever you want to change it again.

**[Download the Windows installer](https://github.com/xiv01/CodexOverlay/releases/download/0.1.0/Codex.Usage.Overlay_0.1.0_x64-setup.exe)**

## Screenshots

<table>
  <tr>
    <th>Settings</th>
    <th>Pinned</th>
  </tr>
  <tr>
    <td><img src="https://i.imgur.com/wMSw4m7.png" alt="Codex Usage Overlay settings" width="480"></td>
    <td><img src="https://i.imgur.com/BecDKrS.png" alt="Codex Usage Overlay pinned" width="480"></td>
  </tr>
</table>

## Prerequisites

- Windows 11
- Node.js 20+ and pnpm
- Rust stable with the MSVC toolchain
- Codex CLI available on `PATH`

Install and authenticate Codex using its supported CLI flow, for example:

```powershell
codex login
codex --version
```

## Development

```powershell
pnpm install
pnpm tauri dev
pnpm test
pnpm tauri build
```

Use mock data without a Codex login:

```powershell
$env:CODEX_OVERLAY_MOCK = "1"
pnpm tauri dev
```

## Use

On first launch the bar opens near the upper center in placement mode. Drag it, then select the pin icon or press Enter. Pinning makes the entire native window click-through, so it cannot interfere with editors, browser tabs, games, scrolling, or text selection.

To regain interaction after pinning, use either:

- `Ctrl+Alt+Shift+U`
- Tray menu: **Edit position**

The tray also provides refresh, details, visibility, optional Windows startup, and quit controls. The app restores its saved location and pinned state while preserving valid negative monitor coordinates. If a monitor is removed, it recovers to an available display.

## Data and Privacy

The Rust backend starts one persistent `codex app-server` child with stdin/stdout JSONL. It initializes once, calls `account/read`, calls `account/rateLimits/read`, listens for `account/rateLimits/updated`, and reconciles every 60 seconds. Sparse updates are merged with the last complete snapshot.

Percentages in the bar are **remaining quota**: `clamp(100 - usedPercent, 0, 100)`. API-key accounts deliberately show usage unavailable because API billing is not ChatGPT subscription quota.

The application never logs or stores email addresses, account IDs, tokens, cookies, raw authentication payloads, or complete process environments. There is no telemetry, analytics, browser automation, or frontend network access.

## Troubleshooting

If the bar reports **Codex not found**, confirm `codex --version` works in a new PowerShell session. Packaged applications can inherit a different `PATH`; add the Codex executable in the future settings surface or launch from an environment with its npm/pnpm/Bun bin directory available.

If it reports **Codex signed out**, run `codex login`, then choose **Refresh now** from the tray. Temporary failures retain the last valid values and mark the connection stale while the app-server reconnects with bounded backoff.

## Build output

`pnpm tauri build` produces the Windows installer under `src-tauri/target/release/bundle/nsis/`.
