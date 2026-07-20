# Repair Resell

## Direction

Repair Resell is a local-first module for building a restoration-funded learning path, not only a flipping tool. Profit matters because it can pay down debt, reduce mortgage pressure, and fund the next project, but the long-term value is the user's growing repair knowledge base.

The module should support this workflow:

```text
find an interesting machine
-> estimate pickup and acquisition cost
-> diagnose symptoms
-> learn how the system works
-> repair or restore it
-> decide whether to keep, sell, part out, or use as a donor
-> record what was learned
```

The long-term arc is to move from low-risk repair candidates toward larger mechanical projects and eventually full vehicle restoration or building a car from the ground up.

## Pickup Economics

Repair Resell should treat pickup logistics as a first-class deal factor. The user works Monday, Tuesday, Thursday, and Friday, with Wednesday, Saturday, and Sunday available for longer pickups. Wednesday is especially useful because many surplus auctions, municipal depots, and business-hour pickups are easier midweek.

The module should eventually model:

- Multi-item auction pickups where fuel and travel time are spread across several lots.
- Regional arbitrage between lower-priced inventory areas and the Kitchener/GTA resale market.
- Estate-auction bundles containing tools, lawn equipment, bicycles, shop equipment, or parts machines.
- Parts-harvesting opportunities, such as buying several non-running machines to repair one, complete another, and sell remaining parts.
- Trailer-enabled hauling for riding mowers, utility trailers, motorcycles, ATVs, compact tractors, and shop machinery.
- Optional return-load opportunities where a truck/trailer trip can offset fuel costs by transporting something back for someone else.

Single-item trips should be treated cautiously when travel costs are high. Multi-item pickups and clustered regional auctions are a better fit for longer drives.

## Learning Progression

Suggested progression:

```text
Phase 1: listings, market observation, low-risk buys, basic repairs
Phase 2: bicycles, push mowers, snow blowers, generators, pressure washers, power tools
Phase 3: riding mowers, utility trailers, motorcycles, ATVs, compact tractors
Phase 4: engine rebuilding, vehicle restoration, full ground-up car build
```

Each phase should teach systems needed for the next one: drivetrain, brakes, fuel, ignition, compression, belts, pulleys, bearings, hydraulics, charging systems, wiring, diagnosis, and parts sourcing.

## Future Competitive Advantage

The module's advantage should come from local history and repeatable evaluation rather than guesswork. Over time, Overlay Forge should help answer:

- Which sources produce worthwhile items.
- Which brands and models are worth watching.
- What similar items actually sold for.
- How often certain failure modes appear.
- Which repairs were profitable, educational, or not worth repeating.
- Whether a long pickup is justified by the whole load, not just one item.

Future automation can include periodic source imports, distance and fuel calculations, preferred-brand alerts, price history, comparable resale history, and model-specific repair history. These must stay backend-owned, local-first, and bounded by the module's scraping and safety rules.

## Repair Knowledge Base

Every repaired item should eventually become a reusable knowledge record. Future repair records should capture:

- Symptoms.
- Diagnosis.
- Root cause.
- Parts used.
- Cost.
- Time spent.
- Photos.
- Manuals and video links.
- Lessons learned.
- Final outcome: kept, sold, parted out, scrapped, or used as a donor.

When the same brand, model, engine family, or failure mode appears again later, the module should surface prior repair notes so the user starts from personal history rather than from scratch.

## Current MVP Boundary

The current data model remains listing-focused: source registry, manual import, conservative source refresh, listing persistence, snapshots, deterministic flags, watchlist, travel profile, and manual estimates.

The active UI is intentionally collapsed to a button-only shell for now. This does not remove or reset the underlying SQLite data.

Inventory, inspections, repairs, parts, sales, analytics, alerts, scheduled imports, transport/load planning, and LLM enrichment remain future work until explicitly implemented.
