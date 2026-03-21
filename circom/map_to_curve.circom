pragma circom 2.2.3;

/*
 * ATLAS Map-to-Curve Relation (Figure 2 of the paper)
 *
 * Verifies (m, (x,y), (k,z)) ∈ R_M2G:
 *   cond_1: x == k + m * T
 *   cond_2: y == z * z
 *   cond_3: y * y == x*x*x - 17
 *
 * Curve: Grumpkin  y² = x³ - 17
 * Tweak bound T = 256 (zkVM) or T = 128 (zkPoS)
 *
 * Inputs:
 *   m - message (public)
 *   x - x-coordinate of curve point (public)
 *   y - y-coordinate of curve point (public)
 *   z - sqrt witness: z² = y (private)
 *   k - tweak witness: x = k + m*T (private)
 *
 * Constraints: 3 multiplication gates
 */

template MapToCurve(T) {
    // public inputs
    signal input m;
    signal input x;
    signal input y;

    // private witnesses
    signal input z;
    signal input k;

    // intermediate signals
    signal x_sq;
    signal x_cu;
    signal z_sq;

    // cond_1: x == k + m * T
    signal mT;
    mT <== m * T;
    x === k + mT;

    // cond_2: y == z * z
    z_sq <== z * z;
    y === z_sq;

    // cond_3: y * y == x*x*x - 17
    x_sq <== x * x;
    x_cu <== x_sq * x;
    signal y_sq;
    y_sq <== y * y;
    y_sq === x_cu - 17;
}
