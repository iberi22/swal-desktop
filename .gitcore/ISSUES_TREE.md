# SWAL Desktop — Wave 1 Dependency Tree

```mermaid
flowchart TD
    subgraph Level 1: Foundation
        SWAL_01["#1 SWAL-01: Theme Engine CLI\n(scripts/swal-theme)"]
        SWAL_02["#2 SWAL-02: Hive Dark Tokens\n(themes/hive-dark.json)"]
        SWAL_03["#3 SWAL-03: Cyber & Nord Themes\n(themes/*.json)"]
    end

    subgraph Level 2: Modules & Integrations (Parallel)
        SWAL_04["#4 SWAL-04: NixOS Node Services\n(nixos/swal-node.nix)"]
        SWAL_05["#5 SWAL-05: Agent Skills Rails\n(skills/swal-theme-creator)"]
        SWAL_06["#6 SWAL-06: A2UI Widget Vault\n(schemas/widget.schema.json)"]
        SWAL_07["#7 SWAL-07: Dashboard SCSS Reactivity\n(eww/eww.scss)"]
        SWAL_08["#8 SWAL-08: Process Monitor & Kill\n(eww/scripts/ram_panel.py)"]
    end

    subgraph Level 3: Deployment & Validation
        SWAL_09["#9 SWAL-09: Node Install Kit\n(scripts/install.sh)"]
        SWAL_10["#10 SWAL-10: E2E Integration Suite\n(tests/test_theme_engine.py)"]
    end

    SWAL_01 --> SWAL_04
    SWAL_01 --> SWAL_05
    SWAL_01 --> SWAL_07
    SWAL_02 --> SWAL_07
    SWAL_03 --> SWAL_07
    SWAL_07 --> SWAL_08
    SWAL_04 --> SWAL_09
    SWAL_07 --> SWAL_10
```

## Merge Order
- **Batch 1 (L1 Foundation)**: SWAL-01, SWAL-02, SWAL-03 (Parallel)
- **Batch 2 (L2 Integrations)**: SWAL-04, SWAL-05, SWAL-06, SWAL-07, SWAL-08 (Parallel with Disjoint File Islands)
- **Batch 3 (L3 Release)**: SWAL-09, SWAL-10 (Sequential Finalization)
