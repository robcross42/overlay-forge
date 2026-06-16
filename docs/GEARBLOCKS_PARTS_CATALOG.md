# GearBlocks Parts Catalog

This one-time reference documents the GearBlocks part categories and cataloged part names currently recognized from the screenshot set in Overlay Forge.

Current catalog metadata:

- GearBlocks game version: `0.8.96622`
- Catalog status: complete
- Validation status: validated

Use this file as shared context for ChatGPT when asking for part identification, build planning, image analysis, or practical physics explanations. The category order follows the GearBlocks left-panel category layout from top left to bottom right.

## Runtime Catalog Image Workflow

Overlay Forge's current source of truth for GearBlocks parts is the runtime API index imported from the Lua exporter logs. The `0.8.96622` catalog is complete and validated, so the normal Parts view hides category image import and clear controls.

When GearBlocks releases a new version and the catalog needs to be refreshed, re-enable the maintenance controls and use this workflow:

1. In GearBlocks, place one of each part in the empty test map.
2. Load `Overlay Forge Construction Exporter` and click `Export All`.
3. Capture the in-game catalog screenshot for the same category.
4. In Overlay Forge, go to Gaming -> GearBlocks -> Parts and select one category filter, such as `Aero`.
5. Click `Import Catalog Screenshot` and choose the in-game catalog screenshot for that category. Overlay Forge refreshes the runtime part index from the current `Player.log`, crops the visible catalog tiles left-to-right/top-to-bottom, and assigns them to that category's runtime API parts in catalog order.
6. The category view renders only the catalog icons in the same fixed eight-column layout used by the game. Click an icon to open that part's detail view.
7. In the detail view, use `Set Image` or `Replace Image` for manual image correction, and use `Notes` for practical observations about the part.

For categories that require scrolling in GearBlocks, import each screenshot separately. Overlay Forge only imports complete visible icon tiles and skips any tile that is cut off by the screenshot edge.

If a category has bad image assignments during a future refresh, select that category and use `Clear Category Images`. This removes the runtime part image associations for that category but does not delete files from disk.

The selected image is copied into:

```text
game-screenshots/gearblocks/part-images/
```

The runtime part record stores both the copied display image path and the original source image path. `AssetGUID` remains the preferred identity key, with `AssetName` and `Category + DisplayName` used as fallbacks.

## Category Order

1. Aero
2. Blocks
3. Bodies
4. Brakes & Clutches
5. Checkpoints
6. Combustion Engines
7. Connectors
8. Control Wheels
9. Electronics
10. Fuel
11. Gears
12. Lights
13. Linear Actuators
14. Motors
15. Pipes
16. Power
17. Props
18. Pulleys
19. Seats
20. Suspension
21. Wheels

## Category Use Guide

| Category | Practical physics use |
| --- | --- |
| Aero | Air movement, thrust, lift, drag, blade direction, pitch, and rotational-speed experiments. |
| Blocks | Rigid frames, brackets, mounting surfaces, spacers, and load paths. |
| Bodies | Driver, passenger, dummy, payload, clearance, mass distribution, and impact-zone reference geometry. |
| Brakes & Clutches | Rotational energy control, drivetrain coupling, decoupling, and staged torque transfer. |
| Checkpoints | Spatial triggers, route validation, target volumes, and path-completion tests. |
| Combustion Engines | Internal-combustion power delivery, crank layout, cylinder layout, airflow, cooling, and torque behavior. |
| Connectors | Pivots, hinges, steering links, rotating shafts, linkages, and constrained mechanical motion. |
| Control Wheels | Manual steering, trim, rotational input, and human control surfaces. |
| Electronics | Control loops, input handling, signal routing, threshold behavior, displays, sensors, and automation logic. |
| Fuel | Fuel storage, range planning, consumable mass, and center-of-gravity changes as fuel is used. |
| Gears | Torque, speed, direction, ratio changes, mechanical advantage, timing, and transmission behavior. |
| Lights | Visual signaling, orientation, status indication, and low-visibility machine-state feedback. |
| Linear Actuators | Controlled straight-line force for steering, lifting, locking, landing gear, and positioning mechanisms. |
| Motors | Electrical-to-rotational motion, servo positioning, stepwise motion, and drivetrain power. |
| Pipes | Fluid or gas routing, manifolds, flow paths, bends, tees, pressure, and packaging. |
| Power | Electrical supply, conversion, storage, harvesting, endurance, and energy placement. |
| Props | Environmental mass, obstacles, payload, collision tests, clearance checks, and stability experiments. |
| Pulleys | Belt routing, torque transfer, speed ratios, differential mechanisms, wrap angle, and belt grip. |
| Seats | Operator location, payload mass, ergonomic reference points, visibility, clearance, and center of gravity. |
| Suspension | Wheel movement, impact absorption, tire contact, camber, toe, ride height, and roll behavior. |
| Wheels | Ground contact, rolling resistance, traction, suspension tuning, load support, stability, and grip. |

## Screenshot Coverage Notes

- `game-screenshots/gearblocks/GearBlocks_20260610_171007_1781125807316_34548.png` confirms the current Suspension entries: control arms, coil-over barrel and piston parts, steering arms, and torsion springs.
- Blue and red material/color appearances in the Suspension screenshot are paint/material variants of the same cataloged part names, not separate part entries.

## Parts By Category

### Aero

- Propeller 3 Blade
- Propeller 3 Blade Reversed

### Blocks

- Angle 3 x 120 Beam
- Angle 5 x 72 Beam
- Angle 9 x 40 Beam
- Angle 120 Beam
- Angle 135 Beam
- Angle 150 Beam
- Angle 157.5 Beam
- Half Rounded Beam
- Beam
- Rounded Beam
- Scaffold Beam
- Block
- Circle Plate
- Cylinder
- Gusset x1
- Corrugated Plate 9x25
- L-Plate
- Labelled Plate
- Plate
- U-Plate
- Sloped Beam
- Sloped Beam Plate
- Sloped Plate
- Sphere
- Offset Tile 1x2
- Offset Tile 2x2
- V 60 Beam
- V 90 Beam
- W 45 Beam
- Wedge Plate

### Bodies

- Dummy Lower Left Arm
- Dummy Upper Left Arm
- Dummy Lower Right Arm
- Dummy Upper Right Arm
- Dummy Head
- Dummy Lower Left Leg
- Dummy Upper Left Leg
- Dummy Lower Right Leg
- Dummy Upper Right Leg
- Dummy Lower Torso
- Dummy Upper Torso
- Male Lower Left Arm
- Male Upper Left Arm
- Male Lower Right Arm
- Male Upper Right Arm
- Male Hair
- Male Head
- Male Lower Left Leg
- Male Upper Left Leg
- Male Lower Right Leg
- Male Upper Right Leg
- Male Lower Torso
- Male Upper Torso
- Racing Helmet

### Brakes & Clutches

- Disk Brake x3
- Disk Brake x4
- Centrifugal Clutch & Ring Gear x3 (24T)
- Centrifugal Clutch x2
- Centrifugal Clutch x3
- Clutch & Ring Gear x3 (24T)
- Clutch & Ring Gear x4 (32T)
- Clutch x3
- Ratchet (Axle to Axle)
- Ratchet (Block to Axle)

### Checkpoints

- Box Checkpoint
- Cylinder Checkpoint

### Combustion Engines

- Engine Crank Nose & Axle
- Engine Rear (Driven) Crank x1 & Axle
- Engine Rear (Driven) Crank x2 & Axle
- Engine Crank x1
- Engine Crank x2
- Engine Cylinder 1x1 0.7L
- Engine Cylinder 1x1 0.7L (Transparent)
- Engine Cylinder 2x2 2L
- Engine Cylinder 2x2 2L (Transparent)
- Engine Head x1
- Engine Head x2
- Engine Throttle x1
- 4-Blade Fan x3
- 7-Blade Fan x4
- Air-cooled Fan x3

### Connectors

- Axle
- Scaffold Axle
- 1-Hole & Axle
- 1-Hole & Plate (H)
- 1-Hole & Slider (H)
- 1-Hole & Plate (V)
- 1-Hole & Slider (V)
- 2-Hole & Axle (Perp)
- 2-Hole & Axle
- 2-Hole & Plate (H)
- 2-Hole & Slider (H)
- 2-Hole & Plate (V)
- 2-Hole & Slider (V)
- Angle 0
- Angle 3 x 90
- Angle 3 x 120
- Angle 4 x 90
- Angle 90
- Angle 120
- Angle 135
- Angle 150
- Angle 157.5
- Angle 180
- Angle Axle 3 x 90
- Angle Axle 3 x 120
- Angle Axle 4 x 90
- Angle Axle 90
- Angle Axle 120
- Angle Axle 135
- Angle Axle 150
- Angle Axle 157.5
- Angle Axle 180
- Angle Limiter (Axle to Axle)
- Angle Limiter (Block to Axle)
- Ball
- Ball & Axle
- CV Joint (Inner)
- CV Joint (Inner) & Axle
- CV Joint (Outer)
- CV Joint (Outer) & Axle
- Knuckle Joint (Inner)
- Knuckle Joint (Inner) & Axle
- Knuckle Joint (Outer 90)
- Knuckle Joint (Outer 90) & Axle
- Knuckle Joint (Outer 180)
- Knuckle Joint (Outer 180) & Axle
- Offset 3-Hole x3
- Offset 3-Hole x5
- Socket (H)
- Socket & Axle (H)
- Socket (V)
- Socket & Axle (V)
- Pin
- 2-Plate & Axle
- Plate & Axle
- U-Joint Yoke & Axle
- Rotor 2
- Rotor 3
- Rotor 4
- Slider Rail

### Control Wheels

- Hand Wheel x3
- Hand Wheel x5
- Sports Steering Wheel x4
- Steering Wheel x4

### Electronics

- Joystick Control
- Lever Control
- Rotary Knob Control
- Slider Control
- 1 Line Display 3x1
- 2 Line Display 3x1
- 2 Line Display 5x2
- 2 Line Display 5x1
- 2 Line Display 9x2
- 4 Line Display 5x2
- 4 Line Display 9x4
- Joypad (Dual Axis)
- Keypad (1 Key)
- Keypad (4 Keys)
- Keypad (9 Keys)
- Edge Detector
- Boolean Operator
- Boolean Multi-Operator
- Pulse Generator
- Boolean Toggle
- Number Calculus
- Number Comparator
- Number Expression
- Number Filter
- Number Formatter
- Number Junction
- Number Multi-Junction
- Number Operator
- Number Register
- Number Selector
- Number Multi-Selector
- PID Controller
- String Selector
- String Multi-Selector
- Timer
- Accelerometer Sensor
- Angle Sensor
- Attitude Sensor
- Contact Pad Sensor
- Distance Sensor 250m
- Distance Sensor 50m
- GPS Receiver
- Inertial Sensor
- Proximity Sensor 100m
- Proximity Sensor 20m
- Speed & Altitude Sensor
- Clock
- Button Switch
- Rocker Switch
- Toggle Switch

### Fuel

- Fuel Tank 9 Litre
- Fuel Tank 70 Litre
- Fuel Tank 375 Litre

### Gears

- Bevel Gear Hi x2 (16T)
- Bevel Gear Hi x3 (24T)
- Bevel Gear Hi x4 (32T)
- Bevel Gear Hi x5 (40T)
- Bevel Gear Hi x6 (48T)
- Bevel Gear Lo x2 (16T)
- Bevel Gear Lo x3 (24T)
- Bevel Gear Lo x4 (32T)
- Bevel Gear Lo x5 (40T)
- Bevel Gear Lo x6 (48T)
- Crown Gear Hi x2 (16T)
- Crown Gear Hi x3 (24T)
- Crown Gear Hi x4 (32T)
- Crown Gear Hi x5 (40T)
- Crown Gear Hi x6 (48T)
- Crown Gear Lo x2 (16T)
- Crown Gear Lo x3 (24T)
- Crown Gear Lo x4 (32T)
- Crown Gear Lo x5 (40T)
- Crown Gear Lo x6 (48T)
- Differential Crown Gear (32T)
- Differential Crown Gear (48T)
- Differential Spur Gear (32T)
- Differential Spur Gear (48T)
- Rack Gear 2-Ball & Slider x3
- Rack Gear 2-Ball & Slider x5
- Rack Gear 2-Ball & Slider x7
- Rack Gear 2-Hole & Slider x3
- Rack Gear 2-Hole & Slider x5
- Rack Gear 2-Hole & Slider x7
- Rack Gear 2-Hole & Slider x13
- Rack Gear x3
- Rack Gear x7
- Spur Gear x1 (8T)
- Spur Gear x1.25 (10T)
- Spur Gear x1.5 (12T)
- Spur Gear x1.75 (14T)
- Spur Gear x2 (16T)
- Spur Gear x2.25 (18T)
- Spur Gear x2.5 (20T)
- Spur Gear x2.75 (22T)
- Spur Gear x3 (24T)
- Spur Gear x3.5 (28T)
- Spur Gear x4 (32T)
- Spur Gear x4.5 (36T)
- Spur Gear x5 (40T)
- Spur Gear x6 (48T)
- Spur Gear x7 (56T)
- Spur Gear x8 (64T)
- Spur Gear x9 (72T)
- Clutch Gear x1 (8T)
- Clutch Gear x1.25 (10T)
- Clutch Gear x1.5 (12T)
- Clutch Gear x1.75 (14T)
- Clutch Gear x2 (16T)
- Clutch Gear x2.25 (18T)
- Clutch Gear x2.5 (20T)
- Clutch Gear x2.75 (22T)
- Clutch Gear x3 (24T)
- Ratchet Gear x1 (8T)
- Ratchet Gear x1.5 (12T)
- Ratchet Gear x2 (16T)
- Ratchet Gear x2.5 (20T)
- Ratchet Gear x3 (24T)
- Worm Gear CCW x1
- Worm Gear CCW x3
- Worm Gear CCW x7
- Worm Gear CW x1
- Worm Gear CW x3
- Worm Gear CW x7

### Lights

- Cone Light x1
- Rectangular Light 1x1
- Rectangular Light 1x2
- Rectangular Light 2x2
- Upright Rectangular Light x1.5
- Upright Rectangular Light x2
- Upright Round Light x1.5
- Upright Round Light x2

### Linear Actuators

- Linear Actuator (Barrel) Large
- Linear Actuator (Piston) Large
- Linear Actuator (Barrel) Large Long
- Linear Actuator (Piston) Large Long
- Linear Actuator (Barrel) Medium
- Linear Actuator (Piston) Medium
- Linear Actuator (Barrel) Small
- Linear Actuator (Piston) Small

### Motors

- Electric Motor Large
- Electric Motor Medium
- Electric Motor Small
- Continuous Servo Motor Medium
- Continuous Servo Motor Small
- Servo Motor Medium
- Servo Motor Small
- Starter Motor Small
- Stepper Motor Medium
- Stepper Motor Small

### Pipes

- Clamped Pipe
- Corner Pipe
- Corner 90 Pipe
- Small Corner 90 Pipe
- Small Corner Pipe
- Cross Pipe
- Small Cross Pipe
- Straight Pipe
- Tee Pipe
- Tee 90 Pipe
- Small Tee 90 Pipe
- Small Tee Pipe

### Power

- Battery 0.5 kWh
- Battery 1.25 kWh
- Battery 2 kWh
- Battery 50 kWh
- Battery 200 kWh
- Alternator Medium
- Solar Panel 9x5
- Solar Panel 15x9
- Solar Panel 25x15

### Props

- Football
- Concrete Traffic Barrier
- Traffic Cone

### Pulleys

- Pulley x1
- Pulley x1.5
- Pulley x2
- Pulley x2.5
- Pulley x3
- Pulley x4
- Pulley x5
- Pulley x6
- Pulley x7
- Differential Pulley x4
- Differential Pulley x6

### Seats

- Car Seat
- Go-kart Seat
- Racing Seat
- Porcelain Throne
- Vintage Seat

### Suspension

- Control Arm 1x4
- Control Arm 1x5
- Control Arm 1x6
- Control Arm 3x5
- Control Arm 3x6
- Control Arm 3x7
- Coil-over (Barrel) Large
- Coil-over (Piston) Large
- Coil-over (Barrel) Medium
- Coil-over (Piston) Medium
- Coil-over (Barrel) Small
- Coil-over (Piston) Small
- Coil-over (Barrel) Small Strong
- Coil-over (Piston) Small Strong
- Coil-over (Barrel) Large
- Coil-over (Piston) Large
- Coil-over (Barrel) Medium
- Coil-over (Piston) Medium
- Coil-over (Barrel) Small
- Coil-over (Piston) Small
- Coil-over (Barrel) Small Strong
- Coil-over (Piston) Small Strong
- Steering Arm 1-Ball 1-Axle x4
- Steering Arm 2-Axle x4
- Steering Arm 2-Ball x4
- Steering Arm 3-Ball x4
- Steering Arm 3x4 A
- Steering Arm 3x4 B
- Steering Arm 3x5 A
- Steering Arm 3x5 B
- Torsion Spring (Axle to Axle)
- Torsion Spring (Block to Axle)

### Wheels

- Aircraft Wheel 2.5x9
- Aircraft Wheel 2x6
- Car Wheel 2.5x6.5
- Car Wheel 2.5x7
- Car Wheel 2x6.5
- Car Wheel 2x7
- Car Wheel 2x8
- Car Wheel 3x6.5
- Car Wheel 3x7
- Car Wheel 3x8
- Car Wheel 4x8
- Go-kart Wheel 2.5x4
- Go-kart Wheel 2x5
- Motorcycle Wheel 1x8
- Off-road Wheel 5.5x11
- Off-road Wheel 5x15
- Off-road Wheel 5x18
- Off-road Wheel 10x16
- Racing Wheel 4x8
- Racing Wheel 5x8
- Trolley Wheel 1x3
- Truck Wheel 2.5x9
- Truck Wheel 3x11

## Suggested ChatGPT Prompt

Use the GearBlocks categories and part names in this catalog as the reference vocabulary. When analyzing a screenshot, identify visible parts by likely name, category, confidence level, and practical physics role. If a part is visually similar to multiple entries, list the closest candidates and explain what visual details would distinguish them.
