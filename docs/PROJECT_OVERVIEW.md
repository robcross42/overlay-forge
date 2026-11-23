# Overlay Forge Project Overview

## Current Shape

Overlay Forge 0.2.1

This is a private development version.

The main app is a resident personal AI host and launcher. Product features and databases remain in their independent repositories.

The host owns:

- A closed typed product registry.
- Requested-target dispatch.
- Last-used product resume state.
- A resident dashboard and compact prompt.
- Host window state.

The host streams multi-turn conversations using the connected ChatGPT plan and stores completed conversations locally. See [AI Anchor Decision and Delivery](AI_ANCHOR.md) for the implementation and remaining gates.

The app does not dynamically load code, execute commands from SQLite, own a marketplace, update products, or contain product-domain tables.

Deferred modules are outside the current release and implementation scope. See
[Future Module Concepts](FUTURE_FEATURES.md) for the proposed `LanInputBridge` Windows module and
the decisions required before it can change the host-only boundary.

## Provenance

The product repositories were extracted from `robcross42/overlay-forge` `develop` commit `96aeb79778e148a66b8f88c2b9bcd27b8a415454`. The original application database remains unchanged as an explicit import and rollback source.
