#!/usr/bin/env python3
"""Add katselin.fi to valkoiset-sivut whitelist."""

from __future__ import annotations

import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WHITELIST = (
    ROOT.parent / "Kotisataman-suljetut-osat" / "valkoiset-sivut" / "whitelist-unified.json"
)


def main() -> None:
    data = json.loads(WHITELIST.read_text(encoding="utf-8"))
    entry = {
        "domain": "katselin.fi",
        "label": "Katselin",
        "category": "other",
        "tags": ["katselin", "kotisatama", "selain", "aloitussivu"],
        "type": "white",
    }
    data["domains"] = [
        d
        for d in data["domains"]
        if not (isinstance(d, dict) and d.get("domain") == "katselin.fi")
    ]
    data["domains"].append(entry)
    data["updated"] = date.today().isoformat()
    WHITELIST.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Added katselin.fi ({len(data['domains'])} domains total)")


if __name__ == "__main__":
    main()
