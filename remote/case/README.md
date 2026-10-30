# 3D model case remote

It should be fairly easy to open the 3d print, and flash new software to the nRF52.

You are allowed to use self tapping screws in M2 and M3.

Wires are going to be soldered directly to pcb to keep it compact.

## Model

- One parametric OpenSCAD file, `remote.scad`, in this directory, tracked in git.
- Build it like `~/Source/real-life-soundboard/case/real-live-soundboard.scad`: measured parts at the
  top, derived values, `assert`s for clearances, a `part` switch (`assembly`, `open`, each printable
  part, `print`) and `%` ghost parts to check the fit.
- Material: PLA.
- Printer: Bambu Lab P1S (256 x 256 x 256 mm bed). Every part prints without supports.
- Tolerances: half of the soundboard's, those were too loose (e.g. `fit` 0.2 instead of 0.4,
  `lid_clr` 0.1 instead of 0.2, `nub_clr` 0.2 instead of 0.4).

## Requirements

- Handheld only: no wall mount, no holder.
- As small as possible, but leave room to solder and route the wires. A slightly larger case beats
  a cramped one.
- The case closes with self tapping screws. Opening the case is how you swap the CR2032.
- Nothing is reachable from outside except the buttons: no USB-C opening, no reset hole. To reflash,
  open the case.
- No on/off switch, the remote is always on and deep sleeps.
- Ignore the board LED, it doesn't need to be visible.
- nRF52840 board is mounted with its components facing up.
- Buttons are soldered with wires straight onto their pins, the same way as the soundboard, with a
  button plate on posts like the soundboard.

## Button layout

- 8 buttons in 2 columns, one column per blind.
- Rows as tight as the soundboard (11 mm pitch).
- The two columns are further apart than the rows, so they read as two separate groups.
- Each column, from top to bottom: calibration, up, custom position, down.
- The calibration button has a larger gap below it than the other rows, so it stands apart from
  the three movement buttons.
- Labels in the lid: ▲ up, ● custom position, ▼ down, ⚙ calibration, and "1" and "2" above the
  columns. Engraved or embossed, whichever prints cleanly on the lid's top face.

## Dimensions:

### nRF52

33.2mm x 17.5mm x 3.3mm

### Buttons

These buttons have pins at the bottom, and the top is not counting the corners of the buttons, the buttons have rubber points (nubs) in the corners.

5.9mm x 5.9mm x 3.3m (height is to the flat top, not nubs)

Nubs:
each corner nub takes:
1.5mm x 1.5mm x 0.5mm

2 pins 3.7mm long positioned at the 2 outer edges in eh center

### CR2032 holder

22.8mm x 18.3mm x 6.3mm (empty)

At the end of the 18.3mm is the opening for the battery. With a cell in it the length grows to
about 24mm.

Pocket, print clearance included: 23.3mm wide, 24.3mm long.

The back of the holder ends in a 13mm wide block, centred, with the pins. A single ground pin sits at
the bottom: the back wall has a 3mm slot in the centre for it, and is 3mm lower than the holder so
the other pins clear it.

### Capacitor (100 µF)

5mm x 5mm body, plus 5mm legs.
