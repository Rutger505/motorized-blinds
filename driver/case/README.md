# 3D model case driver

It should be fairly easy to open the 3d print, and flash new software to the esp.

This 3d model will have to hang across on the outer side of the wall, the layout will be:

Wall -> ~2cm -> bead chain -> this driver module

The module hangs in front of the bead chain, on the room side. The motor shaft points towards the
wall, into the chain. The ~2cm only means the module can't fit between the wall and the chain.

## Model

- One parametric OpenSCAD file, `driver.scad`, in this directory, tracked in git.
- Build it like `~/Source/real-life-soundboard/case/real-live-soundboard.scad`: measured parts at the
  top, derived values, `assert`s for clearances, a `part` switch (`assembly`, `open`, each printable
  part, `print`) and `%` ghost parts to check the fit.
- Material: PLA.
- Printer: Bambu Lab P1S (256 x 256 x 256 mm bed). Every part prints without supports.
- Tolerances: half of the soundboard's, those were too loose (e.g. `fit` 0.2 instead of 0.4,
  `lid_clr` 0.1 instead of 0.2).
- Both blind modules use the same, identical print. No mirrored version.

## Requirements

- The motor sits entirely inside the case.
- The bead chain gear is part of the model: it fits the motor's D shaft and grips the bead chain
  (see "Gear" below).
- Wall mounting: a part is screwed to the wall first, with M6 screws. After that the module's position can still be
  adjusted a little with the print itself (e.g. a slide or rail with detents), without unscrewing
  it from the wall. That adjustment is how the chain gets tensioned and lined up.
- No screws to hold the case or the motor together: only self tapping screws are available, so use
  snap fits, slides or clamping instead. The motor's M3 threaded holes can't take self tapping
  screws.
- The screws into the wall are the only screws.
- Nothing is reachable from outside: no USB-C opening. To reflash the ESP, open the case. The lid
  must be easy to take off.
- The RGB led doesn't need to be visible.
- No vents, the motor only runs in short bursts.
- No endstop.
- Dupont wires stay on the ESP and the TMC2209, so leave room for them. There's no hard size goal.
- 24V input: a cable from the power supply that ends in 2 bare wires. The cable exits at the bottom
  of the case, with strain relief.

## Gear

The chain hangs in a loop from the blind, so it wraps around the bottom of the gear and both
strands go up.

- 8 bead pockets. Bead pitch p = 3.8 + 7.6 = 11.4 mm (bead centre to bead centre).
- Pitch circle (through the bead centres): D = p / sin(180° / 8) = 11.4 / 0.3827 = 29.8 mm.
- The rope runs straight between two beads. The middle of that chord sits at
  D/2 · cos(22.5°) = 13.76 mm from the axis. With the 1.6 mm rope, the groove floor stays at or
  below 13.76 − 0.8 = 12.96 mm, so about 12.9 mm radius.
- Pockets: 3.8 mm beads + clearance, centred on the pitch circle. A rope groove of about
  1.6 mm + clearance wide connects them. It's narrower than the beads, so the beads, not the
  rope, take the pull.
- Flanges on both sides keep the chain on: outer radius about D/2 + bead radius + 1.2 mm =
  14.9 + 1.9 + 1.2 = 18 mm, so the gear is about 36 mm across the flanges. Use 45° flanks
  instead of flat flanges, so the gear prints without supports.
- Chain travel per motor revolution: 8 · 11.4 = 91.2 mm. At 200 full steps per revolution,
  that's 200 · microsteps / 91.2 steps per mm.
- Bore: D shaft, 4.8 mm round, 4.5 mm across the flat.
- The case has a guard around the bottom half of the gear, open at the top where the two strands
  leave, with about one bead diameter of clearance, so beads can't jump out of their pockets.
- The gear presses onto the D shaft, no clamp. `bore_test` prints just the bore to tune `bore_fit`.

## Layout and mounting

- Tall layout: the motor sits at the top, against the left wall, with the TMC2209 next to it. Below
  that is room for the motor plug, then the ESP over the buck module, then the cable entry with
  strain relief.
- Parts: `shell` (side walls and the room facing front, holds the motor and all electronics),
  `back` (the wall facing plate, carries the gear guard and the two mounting beams), `gear`,
  `carriage` and `plate` (the wall plate).
- The back plate snaps into the shell. To open the case, take the whole module off the rail and pull
  the back plate off at the table.
- The motor sits between the shell's front wall and the back plate, with about 0.4 mm of play. Its boss sits in a
  hole in the back plate, so the chain pull goes straight into the back plate.
- The ESP clips out (the DuPont wires stay on). It's flashed outside the case. Its antenna end goes
  under a fixed lip, and the USB end snaps under two flexible hooks.
- The wall plate sits below the gear, with two countersunk M6 screws (12 mm head) and a vertical
  dovetail rail. The rail has 40 mm of travel and 2 mm ratchet teeth: push the module down to
  tension the chain. It can only move up again while the release lever under the carriage is
  pulled.
- Depth: the back plate's two beams slide into tunnels in the carriage. The chain centre can sit
  15 to 30 mm from the wall. One beam has 2 mm ratchet teeth: the case pushes towards the wall
  freely and is held against pulling away. The lever on the side of the carriage releases it.
- No obstacles around the chain, so the size is free.

## Dimensions


### Esp32-c6 (Espressif dev board)


53mm x 28mm x 4.6mm 
DuPont connectors at the bottom plus connected prototyping wire DuPont (not counting cable coming out of connected DuPont): 16.4mm

2 corners at long edge (antenna of dev board):
7mm long
5.3mm wide

These are notches cut out of the PCB, so the antenna end is only 28 − 2 · 5.3 = 17.4mm wide.

### Stepper motor (NEMA 17)

41.8mm x 41.8mm x 39.4mm

The connector is at the back end (away from the shaft). The flat runs the full shaft length.

Corners:
4mm from corner, diagonal edge

Front size with rotating pin:
Center circle:
21.9mm diameter
2mm tall

Rotating pin length from circle:
21mm
4.8mm diameter
Flat edge on pin:
3mm wide
4.5mm 'diameter' from flat edge to other round size

Wire location:
Center of side of motor
Hole 7.5mm x 6.5mm

Screw holes:
3.6mm from sides in corner
3.3mm wide
trims to ~2.2mm, with ridges going out of 2.2mm diameter

### Motor driver (TMC2209 bigtree-tech V1.3)

Top view: 20.4mm x 15.2mm

Heat sink on the back of the board: 12mm. The header pins come out of the other side.

Header pins plus prototyping wires DuPont (not counting wire sticking out of DuPont plastic): 16.4mm


### Buck module (OSKJ LM2596S, 23yTM9cx)

48.9mm x 23.4mm 11.7mm 

### Capacitors

The capacitors are on the TMC2209 module, so the case doesn't need room for a separate one.

### Power cable

Flat 3.5mm x 5.5mm cable from the power supply, ending in 2 wires.

### Bead chain

Beads: 3.8mm diameter.
Rope between the beads: 7.6mm, measured between the bead edges. So the bead pitch is 11.4mm.
Rope diameter: about 1.6mm.
