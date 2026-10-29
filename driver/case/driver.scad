// Driver module enclosure and wall mount. Units: mm.
// Case frame: x along the wall (0 = case centre), y from the back plate's outer
// face towards the room, z up from the case bottom. The wall frame shares x and
// z, with y measured from the wall surface.
// "d" is depth into the shell, measured from the inside of its front wall.

$fa = 2; $fs = 0.4;

part = "assembly";  // "assembly" | "open" | "shell" | "back" | "gear" | "gear_wall" | "gear_back" | "carriage" | "plate" | "bore_test" | "print"
chain_y  = 20;      // preview only: wall to chain centre
vert_set = 20;      // preview only: how far the module has been pushed down the rail
show_wall = true;   // preview only

/* [Measured components] */
mot_w   = 41.8;
mot_l   = 39.4;
mot_ch  = 4;
boss_d  = 21.9;
boss_h  = 2;
shaft_d = 5.0;
shaft_flat = 4.5;          // flat to the opposite round side; the flat itself is 3 mm wide
shaft_l = 21;               // from the boss
plug_w  = 7.5;
plug_l  = 6.5;              // along the motor, at its back end, facing down
plug_room = 12;             // plug body plus the cable bend below the motor

esp_l   = 53;
esp_w   = 28;
esp_t   = 4.6;              // PCB plus components
esp_pcb = 1.6;
esp_pins = 16.4;
esp_notch = [7, 5.3];       // cut from both corners at the antenna end (+x)

tmc_w   = 15.2;
tmc_l   = 20.4;
tmc_hs  = 12;
tmc_pcb = 1.6;
tmc_pins = 16.4;

buck_l  = 48.9;
buck_w  = 23.4;
buck_t  = 11.7;

cable   = [3.5, 5.5];       // flat cable stood on edge: across, depth

bead_d  = 3.8;
rope_l  = 7.6;              // between bead edges
rope_d  = 1.6;

/* [Shell] */
wall     = 2.0;
front_t  = 2.0;
back_t   = 3.0;
corner_r = 4;
fit      = 0.2;
rib      = 1.2;
weld     = 0.5;
rim_t    = 1.0;
lid_clr  = 0.1;
boss_clr = 0.5;             // the motor boss hole printed tight with just fit
ridge    = 0.5;             // snap ridge on the shell tabs; the back plate has a matching groove
tab_w    = 10;
tab_l    = 12;
slot     = 1.0;

/* [Interior] */
esp_gap    = 1;             // ESP components to the buck below
wire_bend  = 6;             // behind the DuPont housings before a wire has to turn
tmc_gap    = 1;             // heat sink off the front wall
cable_zone = 16;
sr_pin_d   = 3;
sr_bite    = 1.0;           // each strain relief pin pushes the cable this far off a straight line
hook_t     = 1.6;
hook_w     = 4;
lip        = 0.8;
buck_rib_h = 4;

/* [Gear] */
pockets     = 8;
bead_clr    = 0.25;
rope_clr    = 0.2;
tooth_rise  = 1.2;          // teeth stand this far past the bead centres
flange_rise = 1.2;          // flange tips past the bead tops
gear_rim    = 2.0;
bore_fit    = 0.2;         // printed holes come out small; 0.1 would not go on
guard_clr   = 1.0;          // flange tips to the guard
guard_t     = 2;
dowel_d     = 1.75;         // a piece of filament aligns the two glued halves
dowel_clr   = 0.15;
dowel_l     = 5;

/* [Mount] */
chain_min = 16;
chain_max = 30;
plate_t   = 3.5;
pw        = 20;
rail_w    = [12, 16];       // at the plate, at the front
rail_d    = 4.5;
ext       = 14;             // plate past the rail ends, for the screws
m6_d      = 6.5;
m6_head   = 12;
travel    = 40;
pitch_v   = 2;
pitch_d   = 2;
tooth_h   = 1.0;
car_h     = 30;
car_t     = 3;
car_slide = 0.3;            // sleeve off the plate face
pawl_w    = 6;
lever_l   = 10;
beam_w    = 10;
beam_h    = 8;              // below the 45 degree roof, which lets the tunnels print
beam_x    = 20;
beam_z0   = 3;
tun_t     = 2.4;
tun_y0    = 1;              // tunnel mouth, from the wall
tun_y1    = 28.5;

// ---------------------------------------------------------------- derived
half_a   = 180 / pockets;
pitch    = bead_d + rope_l;
pitch_r  = pitch / (2*sin(half_a));
chord_r  = pitch_r * cos(half_a);
groove_r = chord_r - rope_d/2 - 0.05;
pocket_d = bead_d + 2*bead_clr;
groove_w = rope_d + 2*rope_clr;
tooth_r  = pitch_r + tooth_rise;
flange_r = pitch_r + bead_d/2 + flange_rise;
chan_hw  = pocket_d/2 + 0.2;
flank_hw = chan_hw + (flange_r - tooth_r);
gear_w   = 2*(flank_hw + gear_rim);
guard_r  = flange_r + guard_clr;
bore_d   = shaft_d + bore_fit;
bore_flat = shaft_flat + bore_fit - bore_d/2;     // flat, from the axis
// Split at the top of the rope groove and printed outer face down, neither half has an
// overhang: the pockets open upwards as bowls.
split_z  = gear_w/2 + groove_w/2;
dowel_r  = (bore_d/2 + groove_r)/2;

shaft_tip = back_t - boss_h - shaft_l;
gear_y0  = shaft_tip;                       // the gear's wall side face
gear_c   = gear_y0 + gear_w/2;

esp_d0   = buck_t + esp_gap;                // ESP component tops
esp_d1   = esp_d0 + esp_t - esp_pcb;        // PCB front
esp_d2   = esp_d0 + esp_t;                  // PCB back, where the pins leave
tmc_d0   = tmc_gap + tmc_hs;
tmc_d1   = tmc_d0 + tmc_pcb;

id       = max(mot_l + 0.4, esp_d2 + esp_pins + wire_bend);
front_in = back_t + id;
od       = front_in + front_t;

tmc_bay  = tmc_w + 2*fit + 2;
iw       = max(mot_w + 2*fit + rib + tmc_bay, esp_l + 2*(fit + hook_t + 1));
ow       = iw + 2*wall;

esp_zc   = wall + cable_zone + esp_w/2;
mot_z0   = esp_zc + esp_w/2 + plug_room + rib + fit;
mot_zc   = mot_z0 + mot_w/2;
ih       = mot_z0 + mot_w + fit - wall;
oh       = ih + 2*wall;

mot_x0   = -iw/2 + fit;
mot_cx   = mot_x0 + mot_w/2;
mot_x1   = mot_x0 + mot_w;
tmc_cx   = (mot_x1 + fit + rib + iw/2)/2;
tmc_zc   = mot_zc;

esp_x0   = -esp_l/2;
esp_x1   = esp_l/2;
esp_tab  = esp_w/2 - esp_notch[1];
esp_pad  = 2;
buck_cx  = esp_x1 - esp_pad - 2*fit - buck_l/2;
cable_x  = -iw/4;

tab_z    = [0.3*oh, 0.7*oh];
bp_hw    = ow/2 - rim_t - lid_clr;

// mount
Fy       = plate_t + rail_d;                // rail front
S        = Fy + fit;                        // sleeve front wall, inside
sleeve_y1 = S + car_t;
tun_z    = tun_t + fit;                     // beam bottom, carriage frame
car_z0   = beam_z0 - tun_z;                 // carriage bottom, case frame
Wi       = beam_x + beam_w/2 + fit;         // ratchet tunnel, outer wall
Wo       = Wi + tun_t;
yb_min   = chain_min - gear_c;              // wall to back plate face
yb_max   = chain_max - gear_c;
beam_l   = yb_min - tun_y0 - 0.5;
arm_root = tun_y0 + 3;
arm_tip  = tun_y1 - 2 - slot;
pawl_y   = arm_tip - 3;                     // depth pawl's locking face, wall frame
arm_z    = [tun_z + 1, tun_z + beam_h - 1];
arm_top  = car_h - 6;
pawl_zt  = 4;
rail_len = car_h + travel + pitch_v;
plate_len = rail_len + 2*ext;
cs_depth = (m6_head - m6_d)/2;

yb       = chain_y - gear_c;

echo(str("case ", ow, " x ", oh, " x ", od, ", gear ", 2*flange_r, " x ", gear_w));
echo(str("chain travel per motor revolution ", pockets*pitch, " mm"));

assert(chain_y >= chain_min && chain_y <= chain_max);
assert(groove_r <= chord_r - rope_d/2);
assert(guard_r < tooth_r + bead_d);                 // a bead can't climb over a tooth past the guard
assert(gear_y0 + gear_w <= -1);
assert(gear_w >= 10);                               // shaft engagement in the bore
assert(dowel_r + dowel_d/2 + 1 < pitch_r - pocket_d/2);
assert(dowel_l/2 + 1 <= gear_w - split_z);
assert(Fy + 1 <= chain_min - gear_w/2);             // rail stays clear of the gear and guard
assert(yb_min - beam_l >= tun_y0 + 0.5);
assert(tun_y1 - (yb_max - beam_l) >= 10);           // beam still well in the tunnel at full depth
assert(tun_y1 + 1 <= yb_min);
assert(sleeve_y1 + lever_l + 1 <= yb_min);
assert(yb_max - pawl_y + pitch_d <= beam_l);        // ratchet teeth stay on the beam
assert(yb_min - pawl_y - pitch_d >= 1);
assert(Wo + 5 <= ow/2 + 5);
assert(pw/2 + 1 <= beam_x - beam_w/2 - fit - tun_t);
assert(tmc_d1 + tmc_pins <= id);
assert(buck_cx - buck_l/2 - fit - rib >= esp_x0 - fit - 0.4);
assert(cs_depth <= plate_t - 0.5);
assert(car_z0 >= 0);
assert(3*ow + 2*pw <= 256);

// ================================================================ helpers
module box(x0, x1, y0, y1, z0, z1) translate([x0, y0, z0]) cube([x1 - x0, y1 - y0, z1 - z0]);

// rounded in x and z, straight along y
module rrect_xz(x0, x1, z0, z1, y0, y1, r)
    translate([0, y1, 0]) rotate([90, 0, 0]) linear_extrude(height = y1 - y0)
        translate([x0 + r, z0 + r]) offset(r = r) square([x1 - x0 - 2*r, z1 - z0 - 2*r]);

function yd(d) = front_in - d;

// a 2D profile in (a, b), extruded along x
module yz(x0, w) multmatrix([[0, 0, 1, x0], [1, 0, 0, 0], [0, 1, 0, 0]]) linear_extrude(w) children();

module beam_2d()
    polygon([[-beam_w/2, 0], [beam_w/2, 0], [beam_w/2, beam_h], [0, beam_h + beam_w/2], [-beam_w/2, beam_h]]);

module rail_2d()
    polygon([[-rail_w[0]/2, plate_t - weld], [rail_w[0]/2, plate_t - weld],
              [rail_w[0]/2, plate_t], [rail_w[1]/2, Fy], [-rail_w[1]/2, Fy], [-rail_w[0]/2, plate_t]]);

module mot_2d(w = mot_w, c = mot_ch)
    polygon([[c, 0], [w - c, 0], [w, c], [w, w - c], [w - c, w], [c, w], [0, w - c], [0, c]]);

module d_bore(h)
    intersection() {
        cylinder(d = bore_d, h = h);
        translate([-bore_d, -bore_d, -1]) cube([bore_d + bore_flat, 2*bore_d, h + 2]);
    }

// post with a lip that snaps over a board's back face; w is inwards, d is depth
module hook_2d(d_back, ledge_d = -1) {
    top = d_back + fit + 2*lip + fit;
    translate([-hook_t, 0]) square([hook_t, top]);
    polygon([[-weld, d_back + fit], [fit + lip, d_back + fit], [-weld, top]]);
    if (ledge_d > 0)
        polygon([[-weld, ledge_d - fit - 1 - weld], [fit + 1, ledge_d], [-weld, ledge_d]]);
}

// ================================================================= shell
module tab_slots()
    for (sx = [-1, 1], tz = tab_z, dz = [tab_w/2, -tab_w/2 - slot])
        box(sx > 0 ? ow/2 - wall - 1 : -ow/2 - 1, sx > 0 ? ow/2 + 1 : -ow/2 + wall + 1,
            -1, tab_l, tz + dz, tz + dz + slot);

module tab_ridges()
    for (sx = [-1, 1], tz = tab_z)
        translate([sx*(ow/2 - rim_t), 0, tz - tab_w/2]) linear_extrude(tab_w)
            polygon([[sx*weld, back_t/2 - ridge], [0, back_t/2 - ridge], [-sx*ridge, back_t/2],
                     [0, back_t/2 + ridge], [sx*weld, back_t/2 + ridge]]);

// The left and top walls hold two sides; the plug leaves through the gap in the bottom rib.
module motor_ribs() {
    y0 = back_t + 1;
    y1 = front_in + weld;
    box(mot_x1 + fit, mot_x1 + fit + rib, y0, y1, mot_z0 + mot_ch, mot_z0 + mot_w - mot_ch);
    for (seg = [[mot_x0 + mot_ch, mot_cx - plug_w/2 - 2], [mot_cx + plug_w/2 + 2, mot_x1 + fit + rib]])
        box(seg[0], seg[1], y0, y1, mot_z0 - fit - rib, mot_z0 - fit);
}

// clips on the short edges, between the two pin rows
module tmc_clips()
    for (sz = [-1, 1])
        multmatrix([[0, 0, 1, tmc_cx - hook_w/2], [0, -1, 0, front_in], [-sz, 0, 0, tmc_zc + sz*(tmc_l/2 + fit)]])
            translate([0, -weld]) linear_extrude(hook_w) hook_2d(tmc_d1 + weld, tmc_d0 + weld);

// The antenna end goes under the fixed lip first, then the USB end snaps under the hooks.
module esp_cradle() {
    for (sz = [-1, 1]) {
        zc = esp_zc + sz*(esp_w/2 - hook_w/2 - 0.5);
        multmatrix([[1, 0, 0, esp_x0 - fit], [0, -1, 0, front_in], [0, 0, 1, zc - hook_w/2]])
            translate([0, -weld]) linear_extrude(hook_w) hook_2d(esp_d2 + weld);
        z0 = esp_zc + sz*(buck_w/2 + fit);
        z1 = esp_zc + sz*(esp_w/2 + 1);
        box(esp_x0 + 0.3, esp_x0 + 4, yd(esp_d1), yd(-weld), min(z0, z1), max(z0, z1));
    }
    z0 = esp_zc - esp_tab + 1;
    z1 = esp_zc + esp_tab - 1;
    box(esp_x1 - esp_pad, esp_x1 + fit + weld, yd(esp_d0), yd(-weld), z0, z1);
    box(esp_x1 + fit, esp_x1 + fit + 2.4, yd(esp_d2 + 2*fit + lip + 1), yd(-weld), z0, z1);
    translate([0, 0, z0]) linear_extrude(z1 - z0)
        polygon([[esp_x1 + fit + weld, yd(esp_d2 + fit)], [esp_x1 - 1, yd(esp_d2 + fit)],
                 [esp_x1 + fit + weld, yd(esp_d2 + 2*fit + lip + 1)]]);
}

module buck_ribs() {
    x0 = buck_cx - buck_l/2 - fit;  x1 = buck_cx + buck_l/2 + fit;
    z0 = esp_zc - buck_w/2 - fit;   z1 = esp_zc + buck_w/2 + fit;
    for (x = [x0, x1], z = [z0, z1]) {
        sx = x == x0 ? -1 : 1;
        sz = z == z0 ? -1 : 1;
        box(min(x, x - sx*6), max(x, x - sx*6), yd(buck_rib_h), yd(-weld), min(z, z + sz*rib), max(z, z + sz*rib));
        box(min(x, x + sx*rib), max(x, x + sx*rib), yd(buck_rib_h), yd(-weld),
            min(z + sz*rib, z - sz*6), max(z + sz*rib, z - sz*6));
    }
}

module cable_hole()
    box(cable_x - cable[0]/2 - fit, cable_x + cable[0]/2 + fit, yd(cable[1] + 2*fit), yd(0), -1, wall + 1);

module strain_relief() {
    off = cable[0]/2 + sr_pin_d/2 - sr_bite;
    for (p = [[off, 4], [-off, 9], [off, 14]])
        translate([cable_x + p[0], yd(-weld), wall + p[1]]) rotate([90, 0, 0])
            cylinder(d = sr_pin_d, h = cable[1] + 2 + weld);
}

module shell() {
    difference() {
        rrect_xz(-ow/2, ow/2, 0, oh, 0, od, corner_r);
        rrect_xz(-iw/2, iw/2, wall, oh - wall, -1, front_in, corner_r - wall);
        rrect_xz(-ow/2 + rim_t, ow/2 - rim_t, rim_t, oh - rim_t, -1, back_t, corner_r - rim_t);
        tab_slots();
        cable_hole();
    }
    tab_ridges();
    motor_ribs();
    tmc_clips();
    esp_cradle();
    buck_ribs();
    strain_relief();
}

// ============================================================ back plate
module guard()
    translate([mot_cx, weld, mot_zc]) rotate([90, 0, 0]) linear_extrude(weld - gear_y0)
        intersection() {
            difference() { circle(r = guard_r + guard_t); circle(r = guard_r); }
            translate([-50, -50]) square([100, 50]);
        }

module beam(s) translate([s*beam_x, weld, beam_z0]) rotate([90, 0, 0]) linear_extrude(beam_l + weld) beam_2d();

// The locking faces point to the room, so the case can't be pulled away from the wall.
module beam_teeth_cut() {
    x5 = beam_x + beam_w/2;
    x4 = x5 - tooth_h;
    y_lo = -(yb_max - pawl_y) - pitch_d;
    y_hi = -(yb_min - pawl_y) + pitch_d;
    translate([0, 0, beam_z0 + 0.5]) linear_extrude(beam_h - 1)
        for (k = [1:ceil((y_hi - y_lo)/pitch_d)]) {
            s = y_lo + k*pitch_d;
            polygon([[x4, s - pitch_d], [x5 + 1, s - pitch_d], [x5 + 1, s], [x5, s]]);
        }
}

module back() {
    difference() {
        union() {
            rrect_xz(-bp_hw, bp_hw, rim_t + lid_clr, oh - rim_t - lid_clr, 0, back_t, corner_r - rim_t - lid_clr);
            guard();
            beam(-1);
            beam(1);
        }
        translate([mot_cx, back_t + 1, mot_zc]) rotate([90, 0, 0]) cylinder(d = boss_d + 2*boss_clr, h = back_t + 2);
        for (sx = [-1, 1]) {
            apex = ow/2 - rim_t - ridge - 0.1;
            g = ridge + 0.1 + lid_clr + 1;
            translate([0, 0, -1]) linear_extrude(oh + 2)
                polygon([[sx*(apex + g), back_t/2 - g], [sx*apex, back_t/2], [sx*(apex + g), back_t/2 + g]]);
        }
        beam_teeth_cut();
    }
}

// ================================================================== gear
module gear() {
    zc = gear_w/2;
    difference() {
        rotate_extrude()
            polygon([[0, 0], [flange_r, 0], [flange_r, zc - flank_hw], [tooth_r, zc - chan_hw],
                     [tooth_r, zc + chan_hw], [flange_r, zc + flank_hw], [flange_r, gear_w], [0, gear_w]]);
        for (k = [0:pockets - 1]) rotate(k*360/pockets)
            hull() {
                translate([pitch_r, 0, zc]) sphere(d = pocket_d);
                translate([tooth_r + 2, 0, zc]) sphere(d = pocket_d);
            }
        rotate_extrude() translate([groove_r, zc - groove_w/2]) square([tooth_r + 1 - groove_r, groove_w]);
        translate([0, 0, -1]) d_bore(gear_w + 2);
        translate([0, 0, -0.01]) cylinder(d1 = bore_d + 1, d2 = bore_d, h = 0.5);
        translate([0, 0, gear_w - 0.49]) cylinder(d1 = bore_d, d2 = bore_d + 1, h = 0.5);
        // asymmetric, so the halves only go together with the bore flats lined up
        for (a = [half_a, 180 - half_a]) rotate(a)
            translate([dowel_r, 0, split_z - dowel_l/2 - 0.5]) cylinder(d = dowel_d + 2*dowel_clr, h = dowel_l + 1);
    }
}

module gear_wall_half() intersection() { gear(); translate([-50, -50, -1]) cube([100, 100, split_z + 1]); }
module gear_back_half() translate([0, 0, gear_w]) rotate([180, 0, 0])
    intersection() { gear(); translate([-50, -50, split_z]) cube([100, 100, 50]); }

module gear_halves() {
    gear_wall_half();
    translate([2*flange_r + 5, 0, 0]) gear_back_half();
}

module bore_test()
    difference() {
        cylinder(d = bore_d + 6, h = 6);
        translate([0, 0, -1]) d_bore(8);
    }

// ============================================================== carriage
// Carriage frame: x and y as the wall frame, z up from the carriage bottom.
module tunnel_2d(grow) offset(delta = grow) beam_2d();

module tunnel(s, grow, y0, y1)
    translate([s*beam_x, y1, tun_z]) rotate([90, 0, 0]) linear_extrude(y1 - y0) tunnel_2d(grow);

module carriage() {
    difference() {
        union() {
            box(-beam_x, beam_x, plate_t + car_slide, sleeve_y1, 0, car_h);
            for (s = [-1, 1]) tunnel(s, fit + tun_t, tun_y0, tun_y1);
        }
        translate([0, 0, -1]) linear_extrude(car_h + 2) offset(delta = fit) rail_2d();
        box(-rail_w[0]/2 - fit, rail_w[0]/2 + fit, plate_t - 1, plate_t + car_slide + 0.1, -1, car_h + 1);
        for (s = [-1, 1]) tunnel(s, fit, tun_y0 - 1, tun_y1 + 1);
        for (sx = [-1, 1])
            box(min(sx*pawl_w/2, sx*(pawl_w/2 + slot)), max(sx*pawl_w/2, sx*(pawl_w/2 + slot)),
                S - 1, sleeve_y1 + 1, -1, arm_top);
        for (z = [arm_z[0] - slot, arm_z[1]])
            box(Wi - 1, Wo + 1, arm_root, arm_tip + slot, z, z + slot);
        box(Wi - 1, Wo + 1, arm_tip, arm_tip + slot, arm_z[0] - slot, arm_z[1] + slot);
    }
    vertical_pawl();
    depth_pawl();
}

// locks against the module moving up; pull the lever towards the room to lift it
module vertical_pawl() {
    d = tooth_h + fit - 0.2;
    yz(-pawl_w/2, pawl_w) polygon([[S + weld, pawl_zt], [S - d, pawl_zt], [S + weld, pawl_zt - d - weld]]);
    box(-pawl_w/2, pawl_w/2, sleeve_y1 - weld, sleeve_y1 + lever_l, 0, 3);
    box(-pawl_w/2, pawl_w/2, sleeve_y1 + lever_l - 2, sleeve_y1 + lever_l, 0, 7);
}

// The lever's underside rises at 45 degrees and stays clear of the wall above the slot.
module depth_pawl() {
    d = tooth_h + fit - 0.2;
    translate([0, 0, arm_z[0]]) linear_extrude(arm_z[1] - arm_z[0])
        polygon([[Wi + weld, pawl_y], [Wi - d, pawl_y], [Wi + weld, pawl_y + d + 0.6]]);
    hull() {
        box(Wo - weld, Wo, arm_tip - 5, arm_tip, arm_z[0], arm_z[1]);
        box(Wo + 4.59, Wo + 4.6, arm_tip - 5, arm_tip, arm_z[0] + 4.6, arm_z[1] + 4.6);
    }
}

// ============================================================ wall plate
// Plate frame: x and y as the wall frame, z up from the plate bottom.
module rail_teeth_cut()
    yz(-pawl_w/2 - 0.5, pawl_w + 1)
        for (k = [0:rail_len/pitch_v - 1]) {
            z = ext + k*pitch_v;
            polygon([[Fy, z], [Fy + 1, z], [Fy + 1, z + pitch_v], [Fy - tooth_h, z + pitch_v]]);
        }

module plate() {
    difference() {
        union() {
            rrect_xz(-pw/2, pw/2, 0, plate_len, 0, plate_t, 3);
            translate([0, 0, ext]) linear_extrude(rail_len) rail_2d();
        }
        rail_teeth_cut();
        for (z = [ext/2, plate_len - ext/2]) translate([0, 0, z]) rotate([-90, 0, 0]) {
            translate([0, 0, -1]) cylinder(d = m6_d, h = plate_t + 2);
            translate([0, 0, plate_t - cs_depth]) cylinder(d1 = m6_d, d2 = m6_head + 0.02, h = cs_depth + 0.01);
        }
    }
}

// ================================================================ preview
module mock_motor() {
    translate([mot_x0, back_t + mot_l, mot_z0]) rotate([90, 0, 0]) linear_extrude(mot_l) mot_2d();
    translate([mot_cx, back_t, mot_zc]) rotate([90, 0, 0]) {
        cylinder(d = boss_d, h = boss_h);
        rotate(90) d_bore(boss_h + shaft_l);
    }
    box(mot_cx - plug_w/2, mot_cx + plug_w/2, back_t + mot_l - 1 - plug_l, back_t + mot_l - 1, mot_z0 - 9, mot_z0);
}

module mock_esp() {
    y0 = yd(esp_d2);
    difference() {
        box(esp_x0, esp_x1, y0, yd(esp_d1), esp_zc - esp_w/2, esp_zc + esp_w/2);
        for (sz = [-1, 1])
            box(esp_x1 - esp_notch[0], esp_x1 + 1, y0 - 1, yd(esp_d1) + 1,
                min(esp_zc + sz*esp_tab, esp_zc + sz*(esp_w/2 + 1)),
                max(esp_zc + sz*esp_tab, esp_zc + sz*(esp_w/2 + 1)));
    }
    box(esp_x0 + 3, esp_x1 - esp_notch[0], yd(esp_d1), yd(esp_d0), esp_zc - 9, esp_zc + 9);
    for (sz = [-1, 1])
        box(esp_x0 + 6, esp_x1 - esp_notch[0] - 2, yd(esp_d2 + esp_pins), y0,
            esp_zc + sz*(esp_w/2 - 1.27) - 1.27, esp_zc + sz*(esp_w/2 - 1.27) + 1.27);
}

module mock_tmc() {
    x0 = tmc_cx - tmc_w/2;
    z0 = tmc_zc - tmc_l/2;
    box(x0, x0 + tmc_w, yd(tmc_d1), yd(tmc_d0), z0, z0 + tmc_l);
    box(x0 + 2, x0 + tmc_w - 2, yd(tmc_d0), yd(tmc_gap), z0 + 3, z0 + tmc_l - 3);
    for (sx = [-1, 1])
        box(tmc_cx + sx*(tmc_w/2 - 1.27) - 1.27, tmc_cx + sx*(tmc_w/2 - 1.27) + 1.27,
            yd(tmc_d1 + tmc_pins), yd(tmc_d1), z0, z0 + tmc_l);
}

module mock_chain()
    translate([mot_cx, gear_c, mot_zc]) rotate([90, 0, 0]) {
        beads = concat([for (k = [0:4]) [pitch_r*cos(-k*45), pitch_r*sin(-k*45)]],
                       [for (j = [1:3], s = [-1, 1]) [s*pitch_r, j*pitch]]);
        for (b = beads) translate([b.x, b.y]) sphere(d = bead_d);
        for (k = [0:3]) hull() for (b = [beads[k], beads[k + 1]]) translate([b.x, b.y]) sphere(d = rope_d);
        for (s = [-1, 1]) hull() for (y = [0, 3*pitch]) translate([s*pitch_r, y]) sphere(d = rope_d);
    }

module mock_inside() {
    %mock_motor();
    %mock_esp();
    %mock_tmc();
    %box(buck_cx - buck_l/2, buck_cx + buck_l/2, yd(buck_t), yd(0), esp_zc - buck_w/2, esp_zc + buck_w/2);
    %box(cable_x - cable[0]/2, cable_x + cable[0]/2, yd(cable[1]), yd(0), -30, wall + 6);
}

module gear_placed() translate([mot_cx, gear_y0, mot_zc]) rotate([-90, 0, 0]) gear();

module assembly() {
    translate([0, yb, 0]) {
        shell();
        back();
        gear_placed();
        mock_inside();
        %mock_chain();
    }
    translate([0, 0, car_z0]) carriage();
    translate([0, 0, car_z0 - ext - (travel - vert_set)]) plate();
    if (show_wall) %box(-60, 60, -4, 0, car_z0 - ext - travel - 20, oh + 60);
}

module shell_print() translate([0, 0, od]) rotate([-90, 0, 0]) shell();
module back_print()  translate([0, 0, back_t]) rotate([-90, 0, 0]) back();
module plate_print() translate([0, plate_len, 0]) rotate([90, 0, 0]) plate();

// ================================================================= render
if (part == "assembly")       assembly();
else if (part == "open")      { shell(); mock_inside(); }
else if (part == "shell")     shell_print();
else if (part == "back")      back_print();
else if (part == "gear")      gear_halves();
else if (part == "gear_wall") gear_wall_half();
else if (part == "gear_back") gear_back_half();
else if (part == "carriage")  carriage();
else if (part == "plate")     plate_print();
else if (part == "bore_test") bore_test();
else if (part == "print") {
    shell_print();
    translate([ow + 10, 0, 0]) back_print();
    translate([2*ow + 20 + pw/2, 0, 0]) plate_print();
    translate([0, oh + 10 + tun_y1, 0]) carriage();
    translate([ow + 10, oh + 15 + flange_r, 0]) gear_halves();
    translate([ow + 10 + 3*flange_r + 20, oh + 15 + flange_r, 0]) bore_test();
}
