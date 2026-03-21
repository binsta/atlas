pragma circom 2.2.3;

include "map_to_curve.circom";
include "multiset_hash.circom";

/*
 * ATLAS Main Circuit
 *
 * Combines map-to-curve and multiset hash verification.
 * Used for zkVM memory consistency checking (Section 5).
 *
 * Proves:
 *   1. Each memory record maps to a valid Grumpkin point
 *   2. All map-to-curve witnesses are valid
 *
 * Security parameters (Section 5.4):
 *   T = 256  (tweak bound)
 *   N = 10   (number of memory records per proof)
 *   |M| <= 2^100 (97-bit records)
 *   Security: >120 bits (Theorem 3)
 */

template ATLASMemoryCheck(N, T) {
    // public inputs
    signal input messages[N];
    signal input xs[N];
    signal input ys[N];

    // private witnesses
    signal input zs[N];
    signal input ks[N];

    // verify all map-to-curve relations
    component msh = MultisetHash(N, T);

    for (var i = 0; i < N; i++) {
        msh.messages[i] <== messages[i];
        msh.xs[i]       <== xs[i];
        msh.ys[i]       <== ys[i];
        msh.zs[i]       <== zs[i];
        msh.ks[i]       <== ks[i];
    }
}

component main {public [messages, xs, ys]} = ATLASMemoryCheck(10, 256);