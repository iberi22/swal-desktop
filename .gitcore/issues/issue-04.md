# SWAL-04: SWAL Node NixOS Module with Xavier & Edge-Mesh Services

## Scope
Create `nixos/swal-node.nix` configuring systemd user services for `xavier` (HTTP `:8006`, MCP `:8100`) and `edge-mesh` (P2P background daemon).

## Acceptance Criteria
- [ ] `swal-node.nix` imported into `configuration.nix`.
- [ ] Environment variables `XAVIER_API_URL` and `SWAL_THEME_DEFAULT` configured.
