// Remote enclosure. Units: mm. Origin: outside bottom-front-left corner.
// +Y points from the battery end towards the calibration buttons.

$fa = 2; $fs = 0.4;

part = "assembly";      // "assembly" | "open" | "tray" | "lid" | "plate" | "print"

/* [Measured components] */
board  = [17.5, 33.2, 3.3];     // Pro Micro nRF52840, components up, long side along Y
board_wire_h = 5;               // wires soldered on top, joints cut flush underneath
board_rib_h  = 2.0;
holder = [22.8, 18.3, 6.3];     // CR2032 holder
cap    = [5, 5, 5];             // 100 uF, loose next to the holder
cap_leg = 5;
btn   = [5.9, 5.9, 3.3];        // base to flat top
btn_nub   = 1.5;
btn_top   = 3.8;                // base to nub tops
btn_act_d = 3.3;
btn_act_h = 4.6;                // base to actuator top
btn_leg   = 3.7;

/* [Shell] */
wall      = 1.6;
floor_t   = 1.2;
top_t     = 1.6;
corner_r  = 4.5;
fit       = 0.2;
rib       = 1.2;
brk_len   = 5;
post_t    = 2.4;
clr_mod   = 0.3;
weld      = 0.5;

/* [Lid] */
rim_t   = 0.8;
lid_clr = 0.1;

/* [Buttons] */
col_pitch = 20;
row_pitch = 11;
cal_pitch = 15;     // calibration row to up row, so calibration stands apart
btn_clr   = 0.1;
plate_t   = 1.2;
plate_lip = 1.2;    // plate reaches past the posts, so the peg holes keep a wall
pkt_drop  = 0.3;    // pocket walls and ribs stay clear of the lid; only the button tops touch it
dish_d    = 10;
dish_min  = 0.4;    // lid thickness at the cross arm tips
act_d     = 4.4;
nub_clr   = 0.2;    // covers button, plate and lid play together
peg_d     = 2.0;
peg_hole  = 2.2;

/* [Labels] */
lab_size  = 3.6;
lab_gap   = 1.0;    // from the dish edge
lab_edge  = 1.6;    // keeps labels off the lid edge
lab_depth = 0.6;
num_size  = 4;
num_font  = "Liberation Sans:style=Bold";

/* [Battery holder] */
clip_w    = 6;
clip_lip  = 0.5;
clip_lead = 0.8;
clip_slot = 0.6;
clip_root = 1.5;    // the slots stop this far above the floor, so the clip flexes as one finger
hold_gap  = 10;     // opening in the rib towards the board, for the wires and a fingertip

/* [Fasteners] */
scr_pilot = 1.7;    // M2 self-tapping
scr_clear = 2.3;
scr_head  = 4.4;
boss_dia  = 2*(corner_r - wall + 0.4);
boss_dia_s = 4.4;   // slimmer where they hang over the battery bay
boss_in   = corner_r - 0.5;     // centre from the outside faces, as close as the countersink allows
boss_deep = 7;
boss_hang = [true, true, false, false];  // front left, front right, rear left, rear right

// ---------------------------------------------------------------- derived
leg_below = btn_leg - plate_t;
cav_h = board.z + clr_mod + board_wire_h + btn_leg + btn.z;

btn_po = btn.x + 2*btn_clr;
pkt_ow = btn_po + 2*rib;
pkt_h  = btn.z - pkt_drop;
slot_l = btn.y + 0.6;

// from the calibration row down: calibration, up, custom, down
row_dy = [0, cal_pitch, cal_pitch + row_pitch, cal_pitch + 2*row_pitch];

lab_dx  = dish_d/2 + lab_gap + lab_size/2;
out_w   = 2*(col_pitch/2 + lab_dx + lab_size/2 + lab_edge);
inner_w = out_w - 2*wall;
col_x   = [inner_w/2 - col_pitch/2, inner_w/2 + col_pitch/2];

hold_pkt   = [holder.x + 2*fit, holder.y + 2*fit];
hold_x     = (inner_w - hold_pkt.x)/2;
hold_lip_z = holder.z + fit;
hold_rib_h = hold_lip_z + clip_lip + clip_lead;
b1_d       = hold_pkt.y + rib;

post_y0  = b1_d;
plate_y0 = post_y0 - plate_lip;
down_y   = plate_y0 + rib + pkt_ow/2;
cal_y    = down_y + row_dy[3];
row_y    = [for (dy = row_dy) cal_y - dy];
plate_y1 = cal_y + pkt_ow/2 + rib;
post_y   = [post_y0, plate_y1 - plate_lip - post_t];

b2_y = post_y[0] + post_t;
b2_d = post_y[1] - b2_y;
board_pos = [(inner_w - board.x)/2, b2_y + (b2_d - board.y)/2];

pil_top = cav_h - btn.z - plate_t;
peg_xy  = [for (x = [brk_len/2, inner_w - brk_len/2], y = post_y) [x, y + post_t/2]];
plate_x0 = 0.3;          plate_x1 = inner_w - 0.3;
plate_rib_x = [col_x[0] - pkt_ow/2 - rib, col_x[1] + pkt_ow/2 + rib];

num_y = cal_y + dish_d/2 + lab_gap + num_size/2;
boss_reach = boss_in - wall + boss_dia/2;     // from the inner wall
inner_l = max(plate_y1 + fit + boss_reach, num_y + num_size/2 + lab_edge - wall);

out_l  = inner_l + 2*wall;
tray_h = floor_t + cav_h;
out_h  = tray_h + top_t;
lid_o  = rim_t + lid_clr;

nub_w    = btn_nub + 2*nub_clr;
nub_c    = btn.x/2 - btn_nub/2;
act_rise = btn_act_h - btn.z;
dish_h   = top_t - dish_min;
cross_w  = 2*(nub_c - nub_w/2);

boss_xy = [[boss_in, boss_in], [out_w-boss_in, boss_in],
           [boss_in, out_l-boss_in], [out_w-boss_in, out_l-boss_in]];
boss_d  = [boss_dia_s, boss_dia_s, boss_dia, boss_dia];
// A hanging boss starts at the bottom of its pilot hole and tapers into its
// corner at 45 degrees or steeper, from a point this far below.
boss_z0   = tray_h - boss_deep;
boss_drop = sqrt(2)*boss_in + boss_dia_s/2;

echo(str("body ", out_w, " x ", out_l, " x ", out_h));

assert(b2_d >= board.y + 2*fit + 2*rib);
assert(board_pos.x - fit - rib >= brk_len);
assert(pil_top >= board.z + clr_mod + leg_below);
assert(plate_rib_x[0] >= peg_xy[0].x + peg_hole/2 + 0.4);
assert(hold_x - rib >= cap.x + 2*fit);
// the taper rises at least 1 mm per mm from its lowest point, the outer corner
assert(boss_z0 - boss_drop + wall + hold_x - rib >= floor_t + hold_rib_h + clr_mod);
assert(clip_lip > fit);
assert(act_rise < top_t);
assert(dish_min >= 0.4);
assert(cross_w >= 2);
assert(dish_d < row_pitch);
assert(col_x[0] + wall - num_size/2 >= boss_in + scr_head/2 + 1);
assert(sqrt(2)*(corner_r - boss_in) + scr_head/2 + 0.5 <= corner_r - lid_o);   // countersink keeps 0.5 of lid corner

// ================================================================ helpers
module rrect(sx, sy, r, h)
    linear_extrude(height = h)
        translate([r, r]) offset(r = r) square([sx - 2*r, sy - 2*r]);

// inner-cavity coordinates
module at(x, y, z = 0) translate([wall + x, wall + y, floor_t + z]) children();

module cavity(extra = 0, drop = 0, grow = 0)
    translate([wall - grow, wall - grow, floor_t - drop])
        rrect(inner_w + 2*grow, inner_l + 2*grow, corner_r - wall + grow, cav_h + extra + drop);

module each_key()
    for (c = [0:1], r = [0:3])
        let($col = c, $row = r) translate([col_x[c], row_y[r]]) children();

// =================================================================== tray
// The hull fills the corner behind the boss, so a slim boss is still tied to the walls.
// A hanging boss leaves the floor free for the battery bay below it.
module boss(p, d, hang) {
    c = [p.x < out_w/2 ? 0 : out_w, p.y < out_l/2 ? 0 : out_l];
    z0 = hang ? boss_z0 : floor_t - weld;
    intersection() {
        hull() {
            translate([p.x, p.y, z0]) cylinder(d = d, h = tray_h - z0);
            translate([min(p.x, c.x), min(p.y, c.y), z0])
                cube([abs(p.x - c.x), abs(p.y - c.y), tray_h - z0]);
            if (hang)
                translate([c.x - 0.005, c.y - 0.005, z0 - boss_drop]) cube(0.01);
        }
        cavity(0, weld);
    }
}

// only at the corners, so wires can leave the pin rows sideways
module board_brackets() {
    x0 = board_pos.x - fit; y0 = board_pos.y - fit;
    w = board.x + 2*fit;    d = board.y + 2*fit;
    h = board_rib_h + weld;
    for (x = [x0 - rib, x0 + w], y = [y0 - rib, y0 + d + rib - brk_len])
        at(x, y, -weld) cube([rib, brk_len, h]);
    for (x = [x0 - rib, x0 + w + rib - brk_len], y = [y0 - rib, y0 + d])
        at(x, y, -weld) cube([brk_len, rib, h]);
}

module plate_posts()
    for (x = [-weld, inner_w - brk_len], y = post_y) {
        at(x, y, -weld) cube([brk_len + weld, post_t, pil_top + weld]);
        at(x < 0 ? brk_len/2 : inner_w - brk_len/2, y + post_t/2, pil_top)
            cylinder(d = peg_d, h = 1.0);
    }

// The lip's underside is 45 degrees, so it prints without supports.
module clip_lip_2d()
    polygon([[-weld, hold_lip_z], [0, hold_lip_z], [clip_lip, hold_lip_z + clip_lip],
             [0, hold_rib_h], [-weld, hold_rib_h]]);

// The holder lifts out on its wires to swap the cell; the clips only keep it from rattling.
// The front wall of the case is the front of the pocket.
module holder_pocket() {
    x0 = hold_x; x1 = hold_x + hold_pkt.x;
    clip_y = hold_pkt.y/2;
    difference() {
        union() {
            for (x = [x0 - rib, x1])
                at(x, -weld, -weld) cube([rib, b1_d + weld, hold_rib_h + weld]);
            for (x = [x0 - rib, x1 + rib - (hold_pkt.x - hold_gap)/2 - rib])
                at(x, hold_pkt.y, -weld)
                    cube([(hold_pkt.x - hold_gap)/2 + rib, rib, holder.z + weld]);
        }
        for (x = [x0 - rib, x1], s = [-1, 1])
            at(x - 1, clip_y + s*(clip_w + clip_slot)/2 - clip_slot/2, clip_root)
                cube([rib + 2, clip_slot, hold_rib_h]);
    }
    at(x0, clip_y + clip_w/2) rotate([90, 0, 0]) linear_extrude(height = clip_w) clip_lip_2d();
    at(x1, clip_y + clip_w/2) mirror([1, 0, 0]) rotate([90, 0, 0])
        linear_extrude(height = clip_w) clip_lip_2d();
}

module tray() {
    difference() {
        rrect(out_w, out_l, corner_r, out_h);
        cavity(top_t + 1);
        translate([rim_t, rim_t, tray_h])
            rrect(out_w - 2*rim_t, out_l - 2*rim_t, corner_r - rim_t, top_t + 1);
    }

    difference() {
        union() {
            for (i = [0:3]) boss(boss_xy[i], boss_d[i], boss_hang[i]);
            plate_posts();
            board_brackets();
            holder_pocket();
        }
        for (p = boss_xy)
            translate([p.x, p.y, tray_h - boss_deep]) cylinder(d = scr_pilot, h = boss_deep + 1);
    }
}

// ================================================================== plate
module btn_pocket()
    difference() {
        translate([-pkt_ow/2, -pkt_ow/2, -weld]) cube([pkt_ow, pkt_ow, pkt_h + weld]);
        translate([-btn_po/2, -btn_po/2, -1]) cube([btn_po, btn_po, btn.z + 2]);
    }

// The ribs around both columns keep the plate from sagging between the post rows.
module plate() {
    rib_w = plate_rib_x[1] - plate_rib_x[0];
    translate([wall, wall, floor_t + pil_top]) difference() {
        union() {
            translate([plate_x0, plate_y0, 0])
                rrect(plate_x1 - plate_x0, plate_y1 - plate_y0, 2, plate_t);
            each_key() translate([0, 0, plate_t]) btn_pocket();
            for (y = [plate_y0, plate_y1 - rib])
                translate([plate_rib_x[0], y, plate_t - weld]) cube([rib_w, rib, pkt_h + weld]);
            for (x = [plate_rib_x[0], col_x[0] + pkt_ow/2 - weld,
                      col_x[1] - pkt_ow/2 - rib + weld, plate_rib_x[1] - rib])
                translate([x, plate_y0, plate_t - weld])
                    cube([rib, plate_y1 - plate_y0, pkt_h + weld]);
        }
        each_key()
            for (s = [-1, 1])
                translate([s*2.6 - 0.8, -slot_l/2, -1])
                    cube([1.6, slot_l, plate_t + 1.01]);
        for (p = peg_xy)
            translate([p.x, p.y, -1]) cylinder(d = peg_hole, h = plate_t + 2);
    }
}

module plate_print()
    translate([-wall - plate_x0, -wall - plate_y0, -floor_t - pil_top]) plate();

// ==================================================================== lid
// The nub holes leave a cross that the button's flat top bears on.
module key_cut() {
    translate([0, 0, -1]) cylinder(d = act_d, h = top_t + 2);
    translate([0, 0, dish_min]) cylinder(d1 = act_d, d2 = dish_d, h = dish_h + 0.01);
    for (sx = [-1, 1], sy = [-1, 1])
        translate([sx*nub_c - nub_w/2, sy*nub_c - nub_w/2, -1])
            cube([nub_w, nub_w, top_t + 2]);
}

module gear_2d(teeth = 8) {
    root = lab_size*0.36;
    tooth_w = lab_size*0.2;
    difference() {
        union() {
            circle(r = root);
            for (i = [0:teeth - 1])
                rotate(i*360/teeth) translate([-tooth_w/2, root - 0.2])
                    square([tooth_w, lab_size/2 - root + 0.2]);
        }
        circle(d = lab_size*0.32);
    }
}

module triangle_2d(up) {
    h = lab_size*0.87;
    scale([1, up ? 1 : -1])
        polygon([[-lab_size/2, -h/2], [lab_size/2, -h/2], [0, h/2]]);
}

// calibration, up, custom, down
module symbol_2d(row) {
    if (row == 0) gear_2d();
    else if (row == 1) triangle_2d(true);
    else if (row == 2) circle(d = lab_size*0.8);
    else triangle_2d(false);
}

module engrave(x, y)
    translate([wall + x, wall + y, out_h - lab_depth])
        linear_extrude(height = lab_depth + 1) children();

module labels() {
    each_key() engrave(0, 0)
        translate([($col == 0 ? -1 : 1)*lab_dx, 0]) symbol_2d($row);
    for (c = [0:1])
        engrave(col_x[c], num_y)
            text(str(c + 1), size = num_size, font = num_font, halign = "center", valign = "center");
}

module lid() {
    difference() {
        translate([lid_o, lid_o, tray_h])
            rrect(out_w - 2*lid_o, out_l - 2*lid_o, corner_r - lid_o, top_t);
        each_key() translate([wall, wall, tray_h]) key_cut();
        labels();
        for (p = boss_xy) translate([p.x, p.y, tray_h - 1]) {
            cylinder(d = scr_clear, h = top_t + 2);
            translate([0, 0, top_t + 1 - 1.1]) cylinder(d1 = scr_clear, d2 = scr_head, h = 1.1 + 0.01);
        }
    }
}

module lid_print() translate([-lid_o, -lid_o, -tray_h]) lid();

// ================================================================ preview
module button_mock() {
    cube(btn);
    for (x = [0, btn.x - btn_nub], y = [0, btn.y - btn_nub])
        translate([x, y, btn.z]) cube([btn_nub, btn_nub, btn_top - btn.z]);
    translate([btn.x/2, btn.y/2, btn.z]) cylinder(d = btn_act_d, h = btn_act_h - btn.z);
    for (s = [-1, 1])
        translate([btn.x/2 + s*2.6 - 0.3, btn.y/2 - 0.4, -btn_leg])
            cube([0.6, 0.8, btn_leg]);
}

module mock() {
    %at(board_pos.x, board_pos.y) cube(board);
    %at(hold_x + fit, fit) union() {
        cube(holder);
        translate([holder.x/2, holder.y/2, holder.z/2]) cylinder(d = 20, h = 3.2, center = true);
    }
    %at(fit, hold_pkt.y - cap.y) union() {
        cube(cap);
        translate([cap.x/2, cap.y, cap.z/2]) rotate([-90, 0, 0]) cylinder(d = 0.6, h = cap_leg);
    }
    each_key() %at(-btn.x/2, -btn.y/2, cav_h - btn.z) button_mock();
}

// ================================================================= render
if (part == "assembly") { tray(); lid(); plate(); mock(); }
else if (part == "open") { tray(); plate(); mock(); }
else if (part == "tray")  tray();
else if (part == "plate") plate_print();
else if (part == "lid")   lid_print();
else if (part == "print") {
    tray();
    translate([out_w + 10, 0, 0]) lid_print();
    translate([2*out_w + 20, 0, 0]) plate_print();
}
