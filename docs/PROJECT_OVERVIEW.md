# Overlay Forge Project Overview

## Current Shape

Overlay Forge 0.3.6

This is a private development version.

The main app is a resident personal AI host and launcher. Product features and databases remain in their independent repositories.

Google sign-in identifies administrator and family sessions. Only the pinned administrator can access the host's existing ChatGPT connection and chat data; family sessions can launch products. Curated family media actions remain future work. See [Google setup](../README.md#google-sign-in-and-family-access).

The host owns:

- A closed typed product registry.
- Requested-target dispatch.
- Last-used product state, without automatic post-sign-in launch.
- A resident dashboard and compact prompt.
- Host window state.

The host streams multi-turn conversations using the connected ChatGPT plan and stores completed conversations locally. See [AI Anchor Decision and Delivery](AI_ANCHOR.md) for the implementation and remaining gates.

The app does not dynamically load code, execute commands from SQLite, own a marketplace, update products, or contain product-domain tables.

Deferred modules are outside the current release and implementation scope. See
[Future Module Concepts](FUTURE_FEATURES.md) for the proposed `LanInputBridge` Windows module and
the decisions required before it can change the host-only boundary.

## Provenance

The product repositories were extracted from `robcross42/overlay-forge` `develop` commit `96aeb79778e148a66b8f88c2b9bcd27b8a415454`. The original application database remains unchanged as an explicit import and rollback source.
