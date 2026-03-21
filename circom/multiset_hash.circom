pragma circom 2.2.3;

include "map_to_curve.circom";

/*
 * ATLAS Multiset Hash Relation (Figure 4 of the paper)
 *
 * Verifies MSH(S) = Σ MapToGroup(mᵢ)
 *
 * For N messages, verifies:
 *   1. Each (mᵢ, xᵢ, yᵢ, zᵢ, kᵢ) ∈ R_M2G
 *   2. digest_x, digest_y = Σ (xᵢ, yᵢ) (group sum)
 *
 * Note: Full elliptic curve group addition in circom requires
 * an EC addition template. Here we verify the map-to-curve
 * witness for each element — the digest check is done
 * by the verifier via the public inputs.
 *
 * Inputs (per message):
 *   messages[N]  - public message array
 *   xs[N]        - public x-coordinates
 *   ys[N]        - public y-coordinates
 *   zs[N]        - private sqrt witnesses
 *   ks[N]        - private tweak witnesses
 */

template MultisetHash(N, T) {
    // public inputs
    signal input messages[N];
    signal input xs[N];
    signal input ys[N];

    // private witnesses
    signal input zs[N];
    signal input ks[N];

    // verify each element satisfies R_M2G
    component mappers[N];

    for (var i = 0; i < N; i++) {
        mappers[i] = MapToCurve(T);
        mappers[i].m <== messages[i];
        mappers[i].x <== xs[i];
        mappers[i].y <== ys[i];
        mappers[i].z <== zs[i];
        mappers[i].k <== ks[i];
    }
}

