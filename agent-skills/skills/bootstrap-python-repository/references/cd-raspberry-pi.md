# Continuous Delivery

## Overview

<project-name> runs as a `systemd` service on a Raspberry Pi on the home network.
Deployment is fully automated: push to `main` → tests pass → Pi is updated and restarted.

The GitHub Actions runner reaches the Pi over [Tailscale](https://tailscale.com),
so no port forwarding or public IP is needed.

---

## How it works

```
Push to main
  → GitHub Actions: lint + type-check + tests
  → Tailscale action: runner joins your Tailscale network
  → SSH into Pi: git pull && uv sync && sudo systemctl restart <project-name>
```

---

## Pi setup (one-time)

### 1. Install uv

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### 2. Clone the repo

```bash
git clone https://github.com/<you>/<project-name>.git ~/<project-name>
cd ~/<project-name>
uv sync
direnv allow
```

### 3. Create the `.env` file

```bash
cp .env.example .env
# fill in your secrets
```

### 4. Install and enable the systemd service

```bash
sudo cp deploy/<project-name>.service /etc/systemd/system/<project-name>.service
sudo systemctl daemon-reload
sudo systemctl enable <project-name>
sudo systemctl start <project-name>
```

Check logs:

```bash
journalctl -u <project-name> -f
```

### 5. Add the Pi to Tailscale

```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
```

Note the Pi's Tailscale hostname — you'll use it in the workflow.

---

## GitHub Actions secrets

| Secret | Description |
|---|---|
| `TAILSCALE_AUTHKEY` | Tailscale auth key (ephemeral, reusable) |
| `PI_SSH_USER` | SSH user on the Pi (e.g. `pi`) |
| `PI_HOSTNAME` | Pi's Tailscale hostname or IP |
| `PI_SSH_KEY` | Private SSH key with access to the Pi |

---

## Workflow

See `.github/workflows/deploy.yml`.

The deploy step only runs on pushes to `main` after all checks pass.
PRs only run lint + tests — no deploy.

---

## systemd service

See `deploy/<project-name>.service`.

The service reads secrets from `~/<project-name>/.env` via `EnvironmentFile`.
It restarts automatically on crash (`Restart=always`).
