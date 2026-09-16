# STATE — Neutron
> Facts only. History: `runs/` (archive). Method: `AGENTS.md` v2.
> **Updated 6 Sep 2026 (Linux box), session 30. GATE4 baseline (WG-frozen
> fix 36427c5): 30-seed ratchet — mean 99.3165%, 12/30 ≥99.5%, 26/30 ≥99.0%.
> 8 Sep s31-s32. s31 561360b + s32 89b851b PUSHED (network recovered):
> infested variants ALIASED to host blocks → ore_infested wrote stone
> over stone. 7 Infested* BlockIds 246-252 + protocol ids. 10101:
> 99.0158% → 99.0368% (−10,895; family 10,328 → 96). Ratchet 424242 +418,
> 12345 +109, 777 flat. s32 89b851b: ancient_city ASSEMBLY ported
> (JigsawPlacement addPieces: random_spread salt 20083232/24/8 golden vs
> ref chunk (-14,9); city_anchor adjust; groundLevelDelta 1; FIFO placer;
> branch free-space AABB carve; ListPoolElement first-child jigsaws).
> s34-s35: ANCIENT CITY — assembly SOLVED, placement BLOCKED on the
> cave-biome gate. 8455ed9: accept-semantics decoded (ONLY_SECOND =
> cand ∧ ¬free ≠ ∅ → vanilla SKIPS; a candidate is accepted only when
> FULLY INSIDE free space; accept carves) → assembly 89/89 ORDER-
> IDENTICAL to the ProbeCityPieces oracle (= ref NBT) on 10101 AND
> 92/93 on 424242 (the 1 = oracle prints LIST:1, same piece). 57 city
> blocks added (probe-dumped protocol ids). Placement wired: 10101
> +8,175 → then 349e33d Beardifier port (BEARD_BOX piece boxes, kernel
> e^{-((dx)²+(dy+0.5)²+(dz)²)/16}, value = -(dyToGround+0.5)*invSqrt(d²
> /2)/2, cell-corner interpolated) → 10101 +43,067 (99.1359%).
> BUT 424242 REGRESSED −122k: I place a city at chunk (-13,9) where the
> REF HAS NONE — the deep_dark biome gate: my noise-biome lookup at the
> stub returns deep_dark, vanilla's = dark_forest (ref section palette
> [dark_forest, deep_dark]; the stub quart cell is dark_forest).
> s37 (abb68f7): ANCIENT CITY + BEARDIFIER WIRED. The s36 "climate
> divergence" was a TEST BUG — the winner-quart probe compared the 10101
> state against a 424242 vanilla probe. With the correct state, neutron's
> climate target matches vanilla bit-for-bit at every probed position
> (10101 stub = deep_dark; 424242 winner quart = dark_forest).
> WIRED: isValidBiome gate (8-corner fiddled voronoi, deep_dark) +
> Beardifier (BEARD_BOX piece boxes, cell-corner interpolated) + assembly
> cache (the jigsaw expansion repeats per chunk in a ±7 radius; without
> it the 524-chunk scan exceeds 40 min).
> Parity 10101: 99.0368% → 99.1410% (+53,878; total −110,402 since the
> city objective started: infested −10,895, palette+assembly +8,175,
> beardifier +53,878 net). Ratchet: 424242 +0 BIT-IDENTICAL (gate
> correctly rejects its potential city at (-13,9)), 777 +0 (no city),
> 12345 +9,531 (gated city placed — vanilla has one there too).
> c5ca14b (s38): junction beard contributions (*0.4) — JigsawJunction
> pairs recorded at accept, contribution with raw unclamped deltas,
> sampled on the cell-corner beard grid (vanilla wraps the whole
> Beardifier inside cacheAllInCell) + affectedBox-24 early-out. 10101
> NEUTRAL (+6 — the *0.4 kernel is small/localized); kept for correctness.
> Parity 10101: 99.1410% / 444,163. Scan runtime 3348s (watch: the city
> adds ~1000s to the 524-chunk scan).
> e7d450a (s39) RE-ATTRIBUTED the 194k air→deepslate family — NOT city
> interiors (ancient_city writer = 0 confusions; placement exact). Family
> spans x -224..-27, y -51..-18, z 0..223 (7 z-bands). Ref NBT verified
> air at (-211,-51,31) and (-223,-51,32) (deep_dark section; sculk in
> palette). EXCLUDED by probes: density (vanilla +0.033 solid), carvers
> (starts 68/68, ELs 3150/3150, no coverage), mineshafts (piece tree
> 147/147 BBs exact; corridors at y -19..-2), city (pieces z ≥ 74), trial
> chambers (zero starts). s40 (46496f6): AQUIFER EXCLUDED too —
> ProbeAquifer substance mode walks the full doFill interpolation and
> calls aquifer.computeSubstance at the cells: density +0.033 solid,
> substance NULL(default)=stone. Structure REFERENCES scan for chunks
> (-14,2)/(-14,1)/(-13,2): zero starts, zero references → the real
> chunk's Beardifier is EMPTY. Every noise/structure mechanism excluded.
> s41 (0f7e6a3): MECHANISM FOUND — SculkVeinBlock.onDischarged
> (SculkVeinBlock.java:81): a vein with no faces left (all faces touch
> sculk) REPLACES ITSELF with AIR (or WATER in fluid). The 194k air→
> deepslate family = sculk-vein discharge air. My port HAS the mechanic
> (sculk/blocks.rs on_discharged → Air) but the census of my chunk
> (-14,2) y=-51: ZERO sculk/veins (8 air, 92 deepslate, 43 tiles) vs ref
> sculk+veins+air — the sculk spread never reaches that section on seed
> 10101. Patch placement (256 attempts/chunk, y uniform -64..256,
> deep_dark voronoi gate) matches vanilla's JSON.
> bfef791 (s42) SPREAD BISECT: attempt trace on chunk (-14,2) origin
> (-224,32) — 28 attempts in y -60..-16, ALL pass the deep_dark gate;
> canSpreadFrom semantics verified identical to vanilla (solid origins
> rejected both sides). Only 3 attempts SPREAD — AIR origins at y
> -20..-24. The ref sculk at y -51..-32 comes from those patches' CHARGE
> CURSORS traveling DOWN 27-31 blocks through solid rock; my cursors
> don't reach — CORRECTION (s43, facf8ec): the replay oracle run
> (ProbeSculkPatch on MY terrain, cave-dump exporter
> NEUTRON_SCULK_CAVEDUMP=1, 20,928 cells + 3 origins) shows vanilla's
> spread on my terrain ALSO stays at y -19/-20. The local origins don't
> reach y=-51 either. The ref sculk at y -51..-32 comes from NEIGHBOR
> origins (deeper attempts or cursor spread into chunk (-14,2)); ref
> chunk (-14,2) has NO sculk at y=-51 (discharged veins became air ✓).
> 0574bab (s44): ALL-ORIGIN BISECT — 550 deep attempts (y -70..-30) pass
> the gate AND spread across 8 origins ((-208,0), (-240,32), (-192,0),
> (-240,0), (-240,16), (-192,64), (-224,48), (-224,0); 29-37 each). REF
> ground truth: a sculk LAYER at y=-52 in chunk (-14,2) with sculk_vein
> at y=-51 (discharged to air). MY chunk: ZERO sculk at any deep y — the
> 550 deep attempts' spreads place no sculk in this chunk. The nearest
> origin attempt (-204,-53,5) is ~19-24 blocks from the ref layer.
> c2f5aa1 (s45): sculk_vein feature WIRED (it was loaded but never
> called — vanilla deep_dark step-7 order: vein idx 0 then patch idx 1,
> each with its own feature seed). Parity 10101 NEUTRAL (444,163 →
> 444,163). The 194k air→deepslate family remains BLOCKED: density,
> aquifer, carvers, mineshaft, city, trial chambers ALL excluded by
> probes (see e7d450a/46496f6); the ref air at y -51..-32 in deep_dark
> sections has no identified mechanism in the noise/structure pipeline.
> 5-iteration cap on this hypothesis REACHED — PARKED.
> c84f4a7 (s47) 424242 LEDGER: the tree family is DISPLACED, not missing
> — tree writer 163,816 (87% border) with INVERTED direction (ref AIR
> where neutron placed dark_oak/pale_oak trees), while the terrain writer
> simultaneously shows ref LEAVES where neutron has air. The trees exist
> in both worlds at DISPLACED positions: vanilla's ticket-order places
> each border tree from a different origin. Net tree-family ≈ 281k cells
> on 424242 — the origin-order cascade is the dominant remaining family.
> 0ed2162 (s50) CONFIRMED with the real ref NBT: chunk (-14,-14) section
> Y=-4 palette = [bedrock, deepslate, gravel, tuff, ores, lava, air] —
> the REAL world has deepslate at y=-63, matching our world. The vanilla
> ProbePreDecorate dump (stone at y=-63) is DEFINITIVELY BROKEN — its
> buildSurface (possibleBiomes={PLAINS} + reflection overload) fails to
> apply the deepslate vertical gradient. The predc1 scene-divergence
> conclusion stands RETRACTED. The displaced-tree family's cause remains
> the origin-order/ticket-sim residual (11-13% violations).
> d493b9e (s51) TREE DISPLACEMENT QUANTIFIED: dark_oak log census —
> REF 164+132=296 vs MINE 135+145=280 across chunks (-14,-14)/(-13,-14):
> trees shifted EAST across the chunk boundary (−29/+13), not uniformly
> over/under-placed. Also confirmed via real ref NBT: (-208,71,-218) =
> AIR — the prior 'vanilla places a tree neutron rejects' was based on
> the BROKEN probe scene. Root cause narrowed: the dark_forest_vegetation
> feature places trees + leaf_litter + sub-features in ONE RNG stream
> (step 9 vegetal); an earlier sub-feature consuming different RNG shifts
> all subsequent tree positions — matching the cross-chunk displacement.
> 5a8c0c6 (s53) MINESHAFTS PER-ORIGIN: vanilla re-runs piece postProcess
> per origin in the decoration loop (setFeatureSeed(dec,1,3), Xoroshiro,
> 3x3 writable-area filter). Old single-pass model carved cave_air the
> ref never had. 424242 98.9475→98.9527% (-27,579). 777 flat; 12345
> +310 (order-model residual at its mineshaft clusters — known cost).
> Mineshaft-overcarve family (cave_air/cobweb/water->clay) on 424242
> window (-1,-8): ELIMINATED (4 gap classes → 0). Lush-clay family grew
> in that window (clay patches read the scene mid-loop; residual =
> ticket-sim order). generate_start memoized.
> f690c35 (s54) TORCH DESYNC: placeSupport else-branch consumes TWO
> maybeGenerateBlock wall_torch rolls (0.05) in vanilla; neutron skipped
> them → every later corridor draw desynced. Fixed + BlockId::WallTorch
> (309) + maybe_generate_block. 424242 98.9527→98.9526 (neutral full
> meter; mineshaft window (-1,-8) −181: stone/tuff/andesite→clay spills
> → 0). 12345 99.1591→99.1616% (−1,276; s53 cost RECOVERED + −966 vs
> s52). 777 bit-identical. Mineshaft RNG stream now draw-exact through
> supports+decor rolls.
> f2647cb (s55) CORRIDOR DECOR + CHUNK CLIP: (1) vanilla structure writes
> are clipped to the DECORATED CHUNK (chunkBB.isInside) — each cell is
> written by exactly ONE origin (its own); neutron wrote region-wide from
> every origin (3x3 filter) → wrong-RNG carves + phantom piece passes
> consuming draws. (2) Corridor postProcess decor ported: spider web
> layer (0.6 inside), 8 cobwebs/section (isInterior short-circuits
> BEFORE the draw), 2 chest rolls (nextInt(100), nextLong only in-chunk),
> spawner roll (nextInt(3); setEntityId draws NOTHING — empty
> WeightedList returns before drawing), rails (0.7 interior/0.9 per z).
> BlockId Cobweb(310)/Rail(311), RegionBuf.height_at.
> 424242 98.9526→98.9645% (−6,123). 12345 −156, 777 −3. ALL SEEDS UP.
> Window (-1,-8): 9,825 → 9,459. Mineshaft-writer cells now draw-exact.
> NOTE: /tmp hit ENOSPC mid-session (12G tmpfs) — one corrupted scan
> (275k artifact) discarded; cache rebuilt.
> 01d543b (s56) TREE STREAM VERIFIED: ProbeVegSeed (new) dumps vanilla's
> raw in_square positions for dark_forest_vegetation (gif 17, step 9).
> MY attempt 0 = (101,34) EXACT vs vanilla — seed formula, lazy-stream
> draw order, nextInt semantics all correct. Ref has NO tree at (101,34)
> → vanilla rejected; my real chain also rejects. Accept gates diverge at
> later attempts where the SCENE differs → displaced trees = lazy-stream
> amplification of terrain diffs. ROOT = terrain family (230k writer
> cells, incl. vanilla-side trees ≈ 125k + tree writer 163k ≈ 289k tree
> family total). Order A/B (z-major 96.08% vs ticket-sim 95.88% on mined
> pairs): parity IDENTICAL on tree window (7,2) — order is NOT the lever.
> Writer map (post-s55): terrain 230k, tree 164k, veg_patch 54k,
> simple_block 37k, block_column 17k, ore 17k — ALL origin/scene cascade.
> 25c2318 (s57) COL-ORDER DISPROVEN (reverted): switched decoration_origin_order
> default ticket_sim → col (z-major, 96.08% mined-pair fit vs sim 95.88%).
> FULL SCAN 424242: 534,440 → 603,055 (+68,615 REGRESSION). The mined-pair
> CSV is a local/partial signal; the full meter disagrees. REVERTED to
> ticket_sim (534,440 baseline restored). NEUTRON_TRUNK_BASES probe mode
> added (diagnostic).
> 6560d62 (s58) LUSH COLUMN DUMP: (-192,13,-93) chunk (-12,-6) —
> REF: y11=air y12-13=MOSS y14+=stone; MINE: y11=cave_vines y12=MOSS
> y13+=stone. Vanilla's patch filled y12+y13; mine filled y12 only and
> kept stone at y13. Patch algorithm verified identical (s6-s13) → the
> pre-patch SCENE differs (my extra solid layer at y13). Lush-clay family
> = patch-fill amplification of scene diffs. Confirms s56/s57: the
> remaining ~534k = scene diffs at border origins; my scene is written by
> the ticket-sim-ordered origin passes whose order violates vanilla's
> real order in 11-13% of mined pairs.
> CAUTION: the earlier trunk_bases ref=0 was a decoder artifact (relative
> path from crate cwd) — probe numbers were always valid.
> 39cb9c6 (s59) MOSS-PATCH WITNESS CELL: (-192,13,-93) traced end-to-end.
> Vanilla density: y9 air, y10-14 SOLID. Vanilla carvers: NONE reach
> y11-13 (new ProbeCarveTrace per-cell probe: CarvingMask membership).
> Ref final: y11 air, y12-13 moss. A depth-1 moss_patch writes ONE cell
> per column → y12+y13 moss = TWO patch passes from DIFFERENT origins
> (border column lx=0). My world: y12 moss only — the second origin's
> patch missed y13. ORIGIN-ORDER CASCADE with a concrete witness.
> The scan (vertical_range 5, 2-air-step + 2-solid-step window) landing
> depends on the pre-patch scene = earlier origins' writes → order.
> bc627f5 (s60) WITNESS CELL MECHANISM: lush_caves has NO moss_patch in
> its biome feature list — the moss = lush_caves_vegetation (= moss_patch,
> count 125, gif 30, depth 1). Depth-1 patch = ONE cell per column →
> y12+y13 moss = TWO origins' passes (border column). My y11
> cave_vines_plant is EXTRA vs vanilla air: my vine hung from my moss;
> vanilla's cave_vines (gif 28) didn't place one. UNRESOLVED: the writer
> of vanilla's y11 AIR over solid density (no carver reaches y11-13; no
> decoration air mechanism found; probes biased: FixedBiomeSource PLAINS
> + ProbeChunkDensity shares lineage with the s50-broken probe). Same
> pattern as the parked s45 sculk family.
> 996d3d1 (s61) SCENE-PARITY INSTRUMENT BUILT + RUN: the pipeline
> decorate_oracle (my NDEC1) → ProbeFullDecorate (real vanilla
> fillFromNoise+buildSurface+applyCarvers, VANILLA_NDEC_OUT re-dump) →
> predecorate_diff --compare-scenes (cell diff + column mode). Witness
> column (-192,*,-93) matches EXACTLY → lush divergence is INSIDE
> decoration (order), confirming s59. Scene diffs in the 5x5 window:
> 221k stone-vs-deepslate + 12k bedrock + 11k dirt/grass = the probe's
> buildSurface artifacts (s50 bug class — NOT real; ref has deepslate at
> y=-63). REAL: 15,711 van=stone mine=air (my carvers carve MORE than
> vanilla's real carvers) + 12,208 stone-vs-water (aquifer). The b81b047
> 'carve bit-exact' verdict was measured against broken probes — needs
> re-verification with a fixed surface path.
> 4c20950 (s62) DECORATION-HEALING QUANTIFIED (SCENE_DIFF_CSV export +
> ledger cross-attribution, 5x5 window (-12,-6)): of 15,711 'van=stone
> mine=air' scene diffs, only 543 survive as final mismatches —
> decoration heals 99.8% of pre-decoration scene diffs
> (last-writer-wins). The 252 'carver-writer' cells are NOT carver
> geometry: they are decor-on-scene (short_grass/moss_carpet/tall_grass/
> azalea/cave_vines) where my scene had air so the decor never grew.
> CARVER GEOMETRY IS NOT THE PROBLEM (s61's 15.7k = healed noise).
> The persistent families (tree 164k, veg_patch 54k, simple_block 37k)
> are the DECORATION diverging — the accept-gate cascade on the differing
> scene, root = origin order (ticket-sim residual 11-13%).
> 9385b81 (s63) ACCEPT_TRACE ORACLE: ProbeFullDecorate now dumps vanilla's
> real per-origin accept/reject for every feature (ACCEPT_TRACE=1). TWO
> fixes: FEATURES_PER_STEP was built from the FIXED-plains possibleBiomes
> (every non-plains feature biome-gate-rejected — the s50 bug class);
> now built from the full overworld biome list. My NDEC1 biome grids
> VERIFIED CORRECT (lush_caves at the witness column; the earlier
> dark_forest read was a decoder section-index error).
> First oracle run (3x3 origins, chunk (-12,-6)): vanilla accepts
> lush_caves_clay at 6 origins, rejects at 3 — per-origin gate results
> now comparable against mine. REMAINING PROBE GAP: gif 27 (ceiling
> vegetation) + gif 30 (lush_caves_vegetation) never attempted at step 9
> despite lush_caves' JSON listing them — holder-identity or indexMapping
> mismatch in the probe, under investigation.
> 1a9fd52 (s64) ORACLE COMPLETE: the s63 'missing gif-27/30' was treeish()
> ROUTING — id.contains("vegetation") captured the moss/clay patches
> (VegetationPatchFeature) and traced them as trees instead of printing
> ACCEPT. Narrowed to exclude (Waterlogged)VegetationPatchFeature. The
> oracle now covers EVERY feature: vanilla's (-12,-6) origin accepts
> lush_caves_clay (gif 29) + lush_caves_vegetation (gif 30) +
> lush_caves_ceiling_vegetation (gif 27) — all three moss/clay writers
> confirmed at the witness chunk. Per-attempt comparison is next.
> bc24bb0 (s65) NEUTRON_LUSH_TRACE: per-attempt oracle for the lush/moss
> patches on my side (MYATTEMPT placed= x= z= y= ok=). Gate bug fixed
> (OnceLock cached the name check with the env check — permanently
> disabled). First measurement (origin (-12,-6), witness window): my
> lush_caves_clay = 65 attempts, ONE accept (-179,-86,-29); vanilla's
> oracle accepted the feature there too. The per-attempt diff needs
> vanilla's per-attempt positions: the treeish branch's SURFACE dump
> (placeGroundPatch's returned surface Set via reflection) IS that
> oracle — s64's treeish narrowing removed patch attempts from the
> tree trace; next iteration re-enable the SURFACE dump for patches
> (keep ACCEPT too) and diff the surface sets.
> 0b78def (s66) PATCHSET ORACLE (partial): reflection chain to
> VegetationPatchFeature.placeGroundPatch BUILT (4 fixes: configured-
> feature unwrap, config field source, TagKey→Predicate wrap, radius
> draws) — the surface set writes to a FILE (stdout's async appender
> swallows the giant lines). KNOWN ARTIFACT: the direct invocation
> BYPASSES the placement modifiers (origin y = raw corner MINY → the
> scan starts in solid deepslate → n=0 for all 9 origins). The faithful
> oracle needs hooking INSIDE place() (instrumented feature class) —
> the modifiers' height_range origin is what the scan needs.
> 07ef0de (s67) PER-ATTEMPT ORACLE COMPLETE: the log-delta PATCHSET
> (placeWithBiomeCheck applies the modifier chain; the ground writes are
> captured from the proxy LOG delta — replaces the s66 bypassed-modifier
> reflection). + NEUTRON_SURFACE_DUMP (my per-patch surface set).
> FIRST DIFF (lush_caves_vegetation, origin (-12,-6)): vanilla patch =
> 406 surface cells; my best-overlapping patch (center (-180,-90)) =
> 100 cells, overlap 82 — MY PATCH IS ~4x SMALLER (radius and/or
> per-column depth). The archived sets: evidence/lush/*.txt (+ my log,
> gitignored — regenerate via NEUTRON_SURFACE_DUMP=1 with
> my_lush_attempts_424242).
> 1ceca39 (s68) ROOT-CAUSE CHAIN CLOSED: per-attempt diff — MY attempt 0
> patch = 197 cells/197 columns (depth 1/cell, moss depth semantics
> correct); vanilla's WHOLE 62-attempt feature = 406 cells/374 columns
> (same depth semantics). Vanilla's attempts average ~6.5 cells; mine
> ~100+. The scan succeeds only on air-above-solid: MY SCENE HAS MORE
> CARVED AIR (s61's 15,711 van=stone mine=air in this window — my
> carvers carve more). The lush patch diff = the carver air diff
> amplified by the environment scan. Fix target: carvers.rs; the s61
> cells are the witness set.
> b89bfe5 (s69) CARVER RESIDUAL MEASURED (carver_edge_dump, 8 chunks):
> open-vs-solid residual = 6,293; FEATURE-FLAGGED = 5,936 (the
> displaced trees/leaf litter); CLEAN carve-edge = 357 cells — TINY.
> The s61 '15.7k' was dominated by tree canopy cells, NOT carver air.
> Carver geometry objective effectively CLOSED. autoexamples=false:
> carver_edge_dump now registered in Cargo.toml.
> OPEN: the lush per-attempt size diff (my attempt 0 = 197 cells vs
> vanilla's 62-attempt total 406) — vanilla's per-attempt sets need
> the oracle to capture attempts individually (the log-delta
> aggregates all 62 attempts in one placeWithBiomeCheck).
> 38223d7 (s70) TREE ORACLE DIFF (tree-family window, chunk (7,2)):
> vanilla ACCEPT_TRACE = 124 dark_forest_vegetation attempts / 42
> accepted across the 9 origins ((6,1): 16/6 at (107,19,71),
> (102,18,72), (100,24,72)). MY per-pass trace: my (x,z) positions
> INCLUDE vanilla's ((107,19),(102,18) present; (100,24) absent) —
> the placement chain + position stream are CORRECT; the LAST-WRITER
> origin differs. FINAL STATE (6,1): vanilla 42 dark_oak trunk columns
> vs mine 44 (near-equal counts), only 2 columns match by position —
> the displaced-tree signature with a clean mechanism: same counts,
> different positions, last-writer origin differs.
> CONCLUSION: the tree family root = the ORDER MODEL (which origin's
> pass is last), confirmed end-to-end. The order model (ticket-sim,
> 95.9% pairwise) residual = the 11-13% inverted pairs.
> PARKED (s70, 5-iter cap): the order model is the sole lever for the
> remaining ~530k cells, confirmed end-to-end (s70 oracle diff: same
> counts, displaced positions). The residual 11-13% inversions are
> ADJACENT-RANK — vanilla's worker-pool completion jitter, likely not
> derivable from the seed (the ref was generated once; the completion
> order depended on machine thread scheduling). Improving the sim
> requires evidence of vanilla's deterministic tie-breaks, if any.
> PARK the lush-clay chain too (s60, needs a faithful-scene probe).
> VERIFIED (s71): session-final full scan 424242 = 98.9645% / 534,440
> (bit-identical to the s55 fix state — the diagnostic commits s56-s70
> introduced no drift). Three-seed ratchet state: 12345 99.1619%,
> 777 99.2797% (verified s64).
> 682e3e6 (s72) ORDER-DEPENDENT GATE IDENTIFIED: the heightmap modifier
> reads the LIVE region state — earlier origins' canopies RAISE the
> OCEAN_FLOOR height for later origins' attempts. Witness (chunk (6,1)):
> column (102,18) my passes accept y=72 = vanilla's exactly (stream +
> heightmap correct); column (107,19) my passes accept y=71 (matches) AND
> y=76 (stacked on earlier canopies). 80/822 accepts (10%) have y>84 =
> stacked trees — the order-model cascade's concrete footprint. Vanilla
> has the same live-heightmap behavior; the divergence is purely the
> LAST-WRITER ORDER.
> 3e7b1c8 (s73) FROZEN OCEAN_FLOOR REVERTED: routing the OceanFloor
> placement modifier through a frozen post-carver snapshot REGRESSED
> the lush window (-12,-6): 17,985 → 21,491 (+3,506). REVERTED to the
> live scan (matches vanilla's tree ys at all three witness columns).
> CORRECTION: the lush-window baseline is 17,985 (97.97%) — earlier
> notes CONFLATED it with the mineshaft window (-1,-8) baseline 9,459.
> The static ChunkStatus read (frozen at CARVERS→FEATURES) contradicts
> the meter — the real WorldGenRegion heightmap lifecycle needs a
> faithful trace. The freeze infra remains (inert, no readers).
> PARKED: order cascade ~530k (jitter); lush chain (needs scene probe);
> carver (357, closed). Session-final baselines: 424242 98.9645% /
> 534,440; 12345 99.1619%; 777 99.2797%.
> 969036b (s78) RING STRIPS CORRECTED: the ref procedure loads FOUR side
> strips after the center square (forceload -192/-161, 160/191, etc.);
> the sim only modeled the WEST strip. Added east/north/south at the
> EXACT script coordinates; footprint halo bounds widened to -14..14.
> Mined-pair consistency unchanged (95.88%); full scan BIT-IDENTICAL
> (534,440 / 98.9645%) — the ring origins' writes don't reach the
> measured region's final state. Batch structure now faithfully mirrors
> the ref procedure's forceload commands.
> PARKED (s78 final): the order cascade ~530k = vanilla's worker-pool
> race floor; the mined-pair CSV cannot rank order models (s76/s77 both
> disproven by the full scan). SESSION BASELINES as above.
> 1638db1 (s79) COMPLETION-ORDER TRACER CAPTURED: vanilla 26.2 has a
> BUILT-IN dispatcher tracer — MC_DEBUG_ENABLED=true +
> MC_DEBUG_VERBOSE_SERVER_EVENTS=true logs RES [x,z] old->new (priority
> resorts) and SUB [x,z] level (pipeline submissions); no agent needed,
> only a log4j2.xml with root DEBUG (the default root filters the
> LOGGER.debug lines). CAPTURED a real server boot (seed 424242, view
> distance 10): 2,116 RES + 647 SUB-at-34, ALL 529 region chunks — the
> boot spawn generation alone produced the entire ref chunk set (the
> forceload square was redundant at view-distance 10).
> OPEN: the naive reconstructions disagree with the mined pairs
> (level-then-FIFO SUB sort = 53.31%; RES promotion = 9 chunks only) —
> the pipeline stages INTERLEAVE across chunks; the decorate execution
> order = the poll order of the DECORATE-specific tasks, not directly
> exposed in the raw RES/SUB lines. Trace archived: evidence/lush/
> trace-boot-424242.log.
> 868729b (s80) TRUE DECORATE ORDER EXTRACTED: ChunkTaskPriorityQueue
> simulated over the MC_DEBUG trace (SUB→level bucket, RES→resort, poll
> = lowest level's first chunk FIFO): 553 polls, 529 chunks at
> level <=34 = the true decorate order (evidence/lush/
> true-decorate-order-424242.txt + the sim replay mode when
> /tmp/true-decorate-order.txt exists).
> PARADOX: the true order vs the mined pairs = 53.50% consistency —
> the REAL server's queue order matches the mined pairs FAR worse than
> my sim (95.88%). EXPLANATION: the mined pairs were inferred by MY OWN
> reconstruction (ProbeDecorate's last-writer proxy), which shares
> assumptions with the sim — the pairs validate the reconstruction,
> NOT the real server. The final blocks still matched at 98.96%
> because decoration heals scene diffs (s62: 99.8%).
> IMPLICATION: the "origin-order cascade" model of the mismatch
> families needs re-examination against the TRUE order — the displaced
> trees/patches may not be order-driven at all despite the same-counts-
> different-positions signature (which the s70 trace showed appears in
> MY passes too).
> 8d9d781 (s81) DEFINITIVE: TRUE vanilla order replayed in my engine →
> tree window (7,2) = 12,794 mismatches vs ticket-sim 11,834 — the
> TRUE order is WORSE. OVER TURNS s70: the order model was NEVER the
> cause of the tree-family mismatches. The displaced-tree signature
> (same counts, different positions; vanilla positions present in some
> passes) + this measurement pinpoints the divergence at the GATES
> (water-depth filter / below-block) or the per-pass scene state at
> the matched positions. NEUTRON_TRUE_ORDER replay mode kept (env-
> gated; was ungated initially — fixed the footprint/default-order
> test failures it caused).
> 8d9d781-followup (s82) LAZY PRIMING REGRESSED TOO: implemented the
> vanilla-faithful lazy per-chunk OCEAN_FLOOR priming
> (getOrCreateHeightmapUnprimed primes on FIRST decoration read; the
> chunk completes CARVERS incl. mineshaft overlap writes by then).
> Tree window: 14,659 mismatches vs live 11,834 — REGRESSED (+2,825).
> THIRD DATA POINT: live scan 11,834 / bulk frozen 12,794-21,491 / lazy
> frozen 14,659 — the LIVE scan matches vanilla best across all
> variants, despite the static decompile reading (ProtoChunk.
> setBlockState updates no heightmaps; ChunkStatus primes FINAL maps at
> CARVERS). VANILLA MECHANISM UNRESOLVED: something updates
> OCEAN_FLOOR during decoration that the decompile doesn't show
> (possibly the per-status task pipeline re-priming, or the chunk
> transitions re-scanning). All three variants archived in this
> session's measurements. REVERTED to live.
> a29fa1c (s83) HEIGHTMAP TRACER CAPTURED — MECHANISM CONFIRMED: the
> decompiled Heightmap.java recompiled with HMSET logging and injected
> into a patched server jar (signatures stripped). 2.8M events captured
> (full trace 322MB, not committed — GitHub limit; witness subset
> evidence/lush/of-trace-6-1.log). DECISIVE: vanilla's OCEAN_FLOOR IS
> LIVE during decoration — witness floors RAISE as earlier origins'
> canopies land: (100,24): 72,72,87; (102,18): 72,72,78,79;
> (107,19): 71,71,72…79 (dark_oak leaves block motion).
> RESOLVES the s73/s82 paradox: the static read was WRONG (the
> heightmaps DO update during decoration through a path the decompile
> doesn't show). My live scan matches vanilla's semantics ✓. The
> displaced-tree residual = the accept-gate interaction with raised
> canopies under a different pass order — the same order-jittered
> surface vanilla itself has.
> PARKED (final): the order cascade ~530k is vanilla's own jitter
> amplified by the live heightmap; no engine change can close it
> without replicating vanilla's thread scheduling exactly.
> BASELINES: 424242 98.9645% / 534,440; 12345 99.1619%; 777 99.2797%.
> NEXT: re-verify baselines periodically. The honest ceiling for
> deterministic parity on this seed is ~98.96% (the residual is
> vanilla's own scheduling nondeterminism, not an engine bug).
> 3194c13 (s74) SMALL-LEDGER AUDIT COMPLETE: huge_mushroom 2,868 (mine
> places brown caps where vanilla has air/leaves; 86% border; the
> 302 red-vs-brown cells are displaced selector outcomes) and seagrass
> 398 (372 displaced attempts + 26 tall/short confusion; the probability
> roll + canSurvive match vanilla line-for-line) — BOTH ARE CASCADE
> CELLS, no feature bugs. Folded into the parked ~535k.
> SESSION STATE (s74): 424242 98.9645% / 534,440; 12345 99.1619%;
> 777 99.2797%. ALL big families traced to the origin-order cascade and
> PARKED with concrete witnesses: trees (s70 displaced positions,
> counts equal), lush patches (s68 scan amplification), small decor
> (s74 displaced selector outcomes). The remaining lever = the order
> model (jitter, likely non-derivable) or a real-server scene probe.
> 95fe7b5 (s75) VIOLATION STRUCTURE QUANTIFIED: the ordering test now
> prints per-model violation offsets. ticket-sim 1,870 violations —
> (0,-1) 778 + (-1,0) 434 (the halo-batch artifact) + (1,0) 298;
> z-major 753 — dominated by (0,-1) 559. Both violate structured
> ORE-spill pairs (~2-4% of mined pairs); the true order has ~0.
> Reconciles s57: z-major fixes ore pairs but regressed the meter
> because the pairs CSV samples ore spills only — full parity depends
> on every feature's last-writer relationship.
> 9c93789 (s76) HALO-BATCH EXPERIMENT: DISPROVEN. Removing the halo
> batch (rect(-12,-12,-11,11)) from the ticket-sim: mined-pair
> consistency ROSE to 96.33% (n=34,103) — but the FULL SCAN CRASHED:
> 1,181,540 mismatches (97.71%) vs baseline 534,440 (+647k REGRESSION).
> The halo batch is ESSENTIAL: it sequences the halo origins' writes that
> the west band depends on. The mined-pair subset LIED again — it
> excludes exactly the cells the halo batch fixes (the n denominators
> also differ per model: with-halo n=45,391, no-halo n=34,103, z-major
> n=19,219 — the models cover different pair subsets, so the percentages
> were never comparable).
> REVERTED (batch toggle removed with the deco_schedule revert).
> ALSO committed: the s73 frozen-vs-live diagnostic test (companion to
> the s73 revert).
> 9s77 spiral hypothesis DISPROVEN: the mined-pair (0,-1) north-late
> violations cluster in the NORTH half (616/778, z<0) — hypothesized a
> spawn-spiral insertion order. Implemented (Batch explicit_coords +
> NEUTRON_DECO_BATCHES=spiral): mined-pair consistency CRASHED to
> 80.83% (17,826 violations). REVERTED. Vanilla's within-rect insertion
> order is neither a rect sweep nor a distance spiral — or the insertion
> order isn't what determines the poll order at all.
> FINAL POSITION: the mined-pair CSV is NOT a valid ranking instrument
> for order models (subset bias, proven twice). The only valid meter is
> the full scan (~35-40 min per order experiment). The parked ~535k
> stands as the honest ceiling for this instrument generation; further
> order-model work requires the faithful-scheduler simulation with the
> full scan as the only judge (~40 min/experiment).
> Prior: S30 DOUBLE BREAKTHROUGH on seed 777 (98.7122% → 99.2797%):
> (1) 55b0f5f surface-rule cave-biome sampled per block (was 8-block cache);
> (2) 0d3093d REMOVED the 'y ≥ min_surface_level−16 → surface_biome'
> shortcut — vanilla SurfaceSystem has no shortcut: context.biome is
> getBiome AT THE BLOCK everywhere. The shortcut masked cave biomes
> extending above min_surface_level−16 (sulfur_caves at y 27-45 on 777),
> blocking the SULFUR_CAVE_GRADIENT bands. Ratchet: 424242 −1,173,
> 12345 −1,799, 456 flat — no regressions. 777 now 99.2797% / 373,153.
> Attributed & parked: trial-chamber tuff_bricks 65k on 777 (structure
> not ported; no vanilla worldgen mechanism outside templates). S29:
> melon predicates (e4e3f1e), PerlinSimplexNoise bit-exact (c766aab).
> Tests green, all pushed.**

## Now

Worldgen 1:1 vs vanilla **26.2**. Meter = `region_parity` + `PARITY_SCAN=1`
+ `PARITY_LEDGER=<csv>`. Ref = canonical 524-chunk world.
| Measurement | Value |
| **GATE6 full 30-seed ratchet** (iceberg extension 505a2ad) | mean **99.4493%**, 15/30 ≥99.5%, 28/30 ≥99.0%, **net −1,070,399 vs gate5, 0 regressions** |
| gate6 movers | 55555 −903,023 (99.5636%) · 123 −167,376 (99.4790%); all other 28 seeds bit-identical |
| seed **424242** (primary) | **98.9704%** / 531,387 (snow + noise_threshold + OFWG fixes) |
| seed **456** | **98.9188%** / 560,137 (stream-desync symptoms only) |
| seed **777** | **99.2797%** / 373,153 |
| seed **55555** | **99.5636%** / 225,209 (berg gap closed) |
| best seed **44444** | **99.8575%** / 73,409 |

**SURFACE-RULE noise_threshold SAMPLING FIXED (16 Sep)**: vanilla
`NoiseThresholdConditionSource` samples 2D noises at `(blockX, 0.0, blockZ)`
and 3D ones at `(blockX, blockY, blockZ)` (SurfaceRules.java:415-436,
592-622). The port sampled **3D for everything except a hard-coded name
allowlist**, so every 2D threshold outside that list read the wrong band —
`powder_snow` (is_3d absent → false) made the snowy_slopes branch fall
through to `snow_block`, which then blocked motion and shifted the
freeze_top_layer column up one cell. Fix: parse `is_3d` from the JSON and
sample y=0 unless it is true. Witness (144,84,-108) seed 424242: raw noise
0.4742 at y=0 (in [0.35,0.6]) vs 0.6087 at y=84 (out) — column now matches
the ref y81-84 powder_snow exactly. 2 regression tests (band witness +
"only sulfur_cave_gradient is 3D" over the parsed datapack rule).

**SNOW FAMILY FIXED (16 Sep)**: `SnowAndFreezeFeature` (freeze_top_layer)
placed NO snow: `biome_climate` read `biome/*` from a never-populated
embedded list → every biome fell back to `(0.5, true)` → `warmEnoughToRain`
was always true. Fix: read the real biome JSONs (temp/precipitation/frozen
modifier), apply `getHeightAdjustedTemperature` (TEMPERATURE_NOISE above
seaLevel+17) + the FROZEN modifier, place `minecraft:snow` (new
`BlockId::SnowLayer`, protocol 6919) at the first-available MOTION_BLOCKING
cell (was top+0 off-by-one), and apply `SnowLayerBlock.canSurvive` tags.
Also un-conflated `BlockId::Snow` (=snow_block, solid/motion) from the
layer across `blocks_motion`, carvers, decorators, multiface, and iceberg
carve. 3 regression tests vs real jar ground truth.

**OCEAN_FLOOR_WG GATE FIXED (16 Sep)**: the ore/disk
`ocean_floor_wg_first_available` read `RegionBuf.heightmaps`, which
`generate_noise_and_surface` fills with a "not air/fluid" scan — the
WORLD_SURFACE predicate, not `blocksMotion`. It counted `powder_snow` (and
other non-motion blocks) as the column top, so `disk_sand`'s
OCEAN_FLOOR_WG placement picked a y one block too high and its
`matching_fluids water` filter then failed. Fix: read the frozen
post-carver snapshot (`ocean_floor_frozen`, `blocksMotion` predicate).
Witness: seed 424242 column (-194,-163) — ref sand y58-61, was dirt; now
matches. A live-read variant was MEASURED and REVERTED (531,943 → 638,330,
+106k): vanilla's heightmaps are per-chunk, so a shared live region map lets
earlier origins' spillover move later origins' gates — the frozen snapshot
is the correct semantics.

Combined: **534,440 → 531,387 (−3,053)**; powder-snow family 1,299 → 4
cells; sand→dirt 419 → 17; freeze_top_layer writer 431 → 323. Ratchet: 12345
window 99.39% bit-identical to baseline; 777 window 99.77% vs 99.78% baseline
(one chunk −49 cells of deterministic coal-ore blob displacement, offset by
+76 fixed cells elsewhere in the same chunk — the ore gate now reads the
frozen post-carver map, see below).

## Closed (git log has full evidence)

- 77b27a2 FallenTreeFeature port (−1,022) · 013a17a replaceable_by_trees
  validTreePos + count-0 streams (−1,334) · 312ed67 bilinear minSurfaceLevel
  (−2,464) · 9d58a2e one-directional steep (−579) · b81b047 carve geometry
  proven bit-exact · 8c22a40 nested-count pipeline (−5,489) · 615443c
  TrapezoidInt / heightmap-parse / canSurvive (−2,671) · 6da2859 wavefront
  ticket sim · f99effe 7x7 window + ref-footprint filter (−14,603) ·
  25c4708 matching_fluids predicate (−320; ratchet improved both seeds).

**Phantom firefly bushes SOLVED (25c4708)**: `matching_fluids` predicate
was missing from `eval_block_predicate` (`_ => true`). Proven: vanilla
trees_birch n=8 ACCEPT (-219,-226,y=68) vs neutron REJECT y=0 at the
identical stream index after 119 matching draws.

**Waterlogged patch interior (PRIMARY, 6 iterations in)**: RADIUS
SETTLED (vanilla place() line 28-29 = `sample(random) + 1`; neutron's
`+1` CORRECT). CONFIRMED: the gif=29 RNG streams for origin (2,9) align
1:1 through draw 290 (per-column rolls identical: neutron's per-column
dump values 0.9608/0.3672/0.8671/0.9499/0.5471... match vanilla's float
sequence exactly). The divergence = vanilla's 291st draw: a vegetation
roll 0.0633 < 0.1 = PASS on ONE extra surface point that neutron's
surface set lacks (neutron 127 vs vanilla 128). The missing point is
the LAST in vanilla's java-HashSet iteration order. Consequence: no
dripleaf at that column in neutron, and base 18+ RNG states diverge.
NEXT (needs java work in ProbeFullDecorate): print the returned
waterSurface set per pool base by reflectively invoking
WaterloggedVegetationPatchFeature.placeGroundPatch (protected, returns
Set<BlockPos>) after replicating the selector boolean + radius draws -
the set diff vs neutron's 127 names the missing column, then examine
that column's scan/below-sturdiness.

**dark_oak boundary objective was a ghost (1 Sep, PROVEN)**: the handoff's
"origins (-224,-240)/(-224,-208) place 0 logs vs vanilla 9/32" came from

`sample(random) + 1`; neutron's `+1` (vegetation.rs:429-430) is CORRECT
and must stay. Streams match through draw 290 (bases, booleans, radius
draws identical; vanilla processes dx=-6..6 same as neutron). The
divergence = ONE extra vanilla float at idx 290 (0.0633 < 0.1 = a
vegetation PASS) → vanilla's base-17 surface set has ONE point neutron
lacks (neutron 127 dumped via NEUTRON_COL_DUMP=39,145 — instrumentation
now committed, env-gated). Next: diff vanilla's base-17 surface (68
water cells + dry clay tops from the capture, bbox x[34..44]
z[140..150]) against neutron's 127 points; examine the missing column's
scan landing. Per-base clay in bbox: neutron 925 vs vanilla 367.
.ndec export, whose chunk (2,8) grid rejected all 62 bases — but (a)
vanilla `BiomeManager.getBiome` and neutron `biome_id_at_block` AGREE
ProbeTreeAttempts' row-major replay, not the ref-world order. Single-pair
reorders are DEAD as a lever.

## Closed (1 Sep s6-s13: lush_caves_clay chain)

- matching_fluids predicate implemented (25c4708, −320; ratchet improved
  both seeds) · radius `+1` SETTLED as correct (vanilla place() line
  28-29 = `sample(random) + 1`) · exposure test PROVEN correct (base-17:
  interior 62 all flooded ≈ vanilla 68) · block_column dispatch VERIFIED
  identical to vanilla source · biome-gate hypothesis DISPROVEN (oracle
  grid artifact; vanilla getBiome and neutron biome_id_at_block AGREE at
  all 12 divergent positions; ref world chunk (2,8) HAS the clay).

## REGRESSION FIXED (2 Sep, s16)

The instrumentation committed in f2cfbcf RESTRUCTURED the corner/edge
skip in place_vegetation_patch (corners fell through; extra_edge==0.0
skipped the column-skip instead of the roll). Caught by the three-seed
ratchet: 424242 regressed to 683,546. FIXED in vegetation.rs — original
logic restored with env-gated logging only; 424242 back to exactly
568,109 / 98.8992% (bit-identical to the pre-instrumentation baseline).
All A/B conclusions drawn from the regressed window (s11-s15) are
INVALID and were re-verified or retracted in the sections above.

Three-seed ratchet (post-fix): 424242 = 568,109 / 98.8992% (unchanged);
12345 = 756,361 / 98.5428% (unchanged); 777 = 717,926 / 98.6142%
(unchanged).

## Standing causal map (1 Sep s14)

**pale_garden short_grass excess = same origin-order mechanism (1 Sep
s18)**: the air→short_grass cells (7431) cluster in pale_garden chunks
(e.g. (0,-2): 124). Vanilla 26.2 pale_garden DOES include
patch_grass_forest (datapack verified; neutron's feature list matches —
glow_lichen at line 73 ✓). The excess = patch surface sets differing by
origin order, same as lush_caves_clay. No independent fix.

**lush_caves_clay attribution CLOSED — mechanism is origin-order
spillover**: the surface-set diff (base 17: vanilla 94 vs neutron 127
points; neutron-only 45 cells on the x=33/z=151 ring) is the decoration
ORIGIN ORDER mechanism, not a patch-code bug. Proof: base (39,84,145)'s
ring columns flood in neutron but not vanilla because their ground
placement depends on prior origins' spillover (earlier patches filled
the floor), which differs between vanilla's real order and neutron's sim
order. ALL patch internals verified identical to vanilla source (radius
+1, depth loop incl. same-block skip, exposure test, block_column).
The lush_caves_clay divergence is a downstream symptom of the
border-zone/order divergence — same root as the tree gap. Lever remains
the origin order model (part of the 87%-border cluster).

Tree-gap attribution: **87-89% of tree-gap cells sit in the chunk BORDER
zone**; 350 chunks affected. Remaining writers: vegetation_patch 59k,
simple_block 38k, ore 18k, block_column 18k.

**Per-base evidence chain CLOSED (2 Sep s17)**: NEUTRON_PATCH_LOG now
logs every place_vegetation_patch call (base position + variant). For
origin (2,9): bases 0-16 have IDENTICAL positions and verdicts in both
sides (base 17 = (39,84,145) pool, accepted in both); bases 18+ diverge
because vanilla's 291st draw (vegetation PASS 0.0633 on its 128th
surface point in java-hash order) shifts the RNG state. The 28-point
surface deficit in neutron's base 17 (100 vs ~128) traces to prior
origins' spillover at those columns — the origin-order mechanism,
confirmed at per-column granularity. All patch internals verified vs
vanilla source. The chain is fully attributed: origin order is the sole
root cause of the lush_caves_clay divergence.

## Next

0. **GATE6 COMPLETE (8 Sep s31)**: all 30 gate seeds re-measured after the
   iceberg extension. net **−1,070,399** vs gate5, ZERO regressions. mean
   99.4493% (was 99.3802), 15/30 ≥99.5% (was 14), 28/30 ≥99.0% (was 27).
   Only 55555 (−903,023) and 123 (−167,376) moved — the extension only
   affects frozen-ocean columns; the other 28 seeds are bit-identical,
   confirming the port is exactly scoped. Below 99.0 remain 456 (98.92)
   and 424242 (98.95) — both pure border/origin-order cascade.
0.5. **FROZEN-OCEAN BERG EXTENSION PORTED (7 Sep s31, commit 505a2ad)**:
   SurfaceSystem.frozenOceanExtension now paints snow_block/packed_ice berg
   columns (three iceberg NormalNoises + per-column noiseRandom + FROZEN
   temperature-modifier melt check via the newly-extended bit-exact
   PerlinSimplexNoise). 55555: −903,023 (97.81 → 99.5636%, no longer the
   worst seed); 123: −170,174 (99.479%); 424242 bit-identical. The parked
   55555 iceberg chain is CLOSED. NEXT: re-run the 30-seed gate (gate6) —
   55555 and 123 both improved; the remaining sub-99.5 seeds are 456/424242
   (border cascade, structural) — mean should rise ~0.06pp to ~99.44.
0.5. **GATE5 COMPLETE (7 Sep s30)**: all 30 gate seeds re-measured after the
   two surface-rule biome fixes. net **−990,130** cells vs gate4, ZERO
   regressions. mean 99.3802% (was 99.3165), 14/30 ≥99.5% (was 12),
   27/30 ≥99.0% (was 26). Every seed improved or flat; the fixes are
   universal (per-block getBiome semantics), not seed-specific.
   POST-GATE5 LEDGERS (271af41): 424242 = dark_oak/pale_oak canopy border
   cascade (87% border, dispersed — no new writer). 456 = missing jungle
   trees 207k (leaves/log/vine -> air; mega-jungle port regression was
   attributed to selector-stream desync BEFORE the surface fixes —
   re-land + measure is the next candidate) + genuine terrain surface
   diffs 9.5k (clay->deepslate 4.5k lush-cave floors y<-16;
   dirt->grass_block 2.9k jungle floor). Self-contained regression tests
   for the column (-88,-56) fix landed (sulfur_column_biomes,
   sulfur_pipeline_fixed).
   MEGA-JUNGLE RE-LAND TESTED AND REVERTED (37ff781, 7 Sep s31): full
   draw-exact port (giant 2x2 core + branch loop + jungle foliage) on
   seed 456: 560,137 -> 626,695 (+66,558 REGRESSION); 424242 bit-identical.
   SECOND confirmation of the selector-stream desync failure mode (789
   mangrove was first): no-op Unknown trunks absorb upstream desync;
   consuming real draws shifts later attempts. Do NOT re-land any tree
   port until the per-origin step-chain desync (border cascade) is fixed.
   Note recorded at the TrunkKind dispatch site in tree/cfg.rs.
   456 LUSH-CLAY OBJECTIVE OPENED (c5d713c): 4,470 vanilla clay cells
   (lush-cave floors y -32..0, e.g. (-118,-18,-51)) where neutron keeps
   deepslate. Biomes AGREE (lush_caves both sides, probed). The
   lush_caves_clay patch (step 9, count=62, env-scan down 12 + biome
   gate) does not fire in neutron for those chunks; patch internals were
   verified identical at 424242 origin (2,9). CLEARED (004bcd3): the
   origin 3x3 union DOES include lush_caves for all three chunks; the
   missing clay is actually ORE_CLAY (step 6, size-33 blob, count=46,
   biome-gated), not the vegetation patch. Y-band evidence: vanilla
   missing clay y -32..0; neutron-only clay leaks to +56 — per-attempt
   in_square/height draws shifted by the upstream origin-order stream
   desync (same root as the tree cascade). No local fix; the desync
   remains the sole root lever for 456 (trees 207k + clay 4.5k).
   ORIGIN-ORDER MODEL CLOSED AT CEILING (c2eb1ef, 7 Sep s31): on the mined
   45,391-pair CSV, no simple within-batch ordering beats the ticket_sim
   arm (best alternative 96.08% interior vs sim 95.88/96.94). All 2,151
   violations are WEST/NORTH winner-displacement only — worker-completion
   jitter consistent with vanilla's 0.85% race floor. The 95.9% ceiling
   is the practical limit of deterministic ordering; advancing the desync
   lever requires a completion-order tracer (javaagent).
   DRIPSTONE STREAM DIFF OPENED (d56b893, 8 Sep s31): seed 10000 chunk
   (-14,-7) origin (-224,-112) — dripstone_cluster draw COUNTS match
   exactly (vanilla GIFDRAW 151 == neutron 151), gif mapping pinned
   (index 4), seeding path re-verified vs ChunkGenerator.java — yet the
   first-attempt POSITIONS differ (vanilla origin+(2,5) vs neutron
   (-218,23,-100) = origin+(6,12)). Same count, different values: the
   divergence is inside the feature's draw VALUES (or the GIFDRAW list
   interleaves the count/height draws differently than assumed — the
   first draws don't fit the in_square pattern). NEUTRON_ICE_LOG now
   dumps attempt positions + draw counts; DECODED (945da54, 8 Sep s31):
   vanilla GIFDRAW values are RAW nextInt results — [2,5,14,...] = count
   raw 2 (uniform 48..96 -> 50 attempts) + per attempt (x,z,y). Neutron
   consumes IDENTICAL 151 draws for the same origin (stream aligned
   draw-for-draw; decorationSeed + feature(4,7) first draws pinned in
   feature_rng.rs test vs ProbeDecoSeed). The ref's STORED section
   biomes at the attempt cells are [forest, plains] — no dripstone_caves
   — so vanilla's BiomeFilter REJECTS all 50 attempts at this origin,
   same as neutron. The ref's dripstone here is spill-in from NEIGHBOR
   origins; the symmetric displacement is the stored-biome border
   cascade (same family as the S30 per-block finding). No feature bug;
   dripstone ~540k cells across 6 seeds joins the cascade total.
1. **PER-BLOCK BIOME FIX LANDED (6 Sep s30, commit 55b0f5f)** — root cause
   of the 777 sulfur-family gap. apply_surface_rules cached the cave-biome
   sample every 8 blocks; vanilla evaluates BiomeManager.getBiome per
   block. Thin bands (sulfur_caves 2-3 blocks inside birch_forest) got the
   wrong biome, so the SULFUR_CAVE_GRADIENT sulfur/cinnabar surface rule
   never fired. 777: 667,156 → 551,047 (−116,109; 98.7122% → 98.9363%);
   424242/456/12345 bit-identical. Proof chain: vanilla-vs-neutron voronoi
   agree 300/300 underground + 399/399 near-surface sites (block-level);
   ProbeClimateAt quart-coord regression FIXED (S28 fix had been lost from
   the probe on disk) — climate exact; 22 "classifier mismatches" were a
   pure-vs-voronoi probe artifact, RETRACTED. FOLLOW-UP (0d3093d): post-fix
   re-ledger showed 190k sulfur/cinnabar cells still -> stone at y 27-45 —
   ABOVE min_surface_level-16, where a second shortcut substituted the
   surface biome. Removed: vanilla has no shortcut, getBiome per block
   everywhere. 777 total: −294k, now 99.2797%. Ratchet: 424242 −1,173,
   12345 −1,799, 456 flat. Remaining on 777: trial-chamber tuff_bricks
   65k (structure unported, PARKED like ancient_city; no vanilla worldgen
   mechanism outside the trial_chambers template pool), trees 63k, ore
   82k (border cascade). NEXT: re-run 30-seed gate — the shortcut removal
   may move every seed with cave biomes above min_surf-16.
2. **GATE4 30-SEED RATCHET COMPLETE (6 Sep s29, after WG-frozen fix
   36427c5)**: all 30 gate seeds measured. mean 99.3165%, 12/30 ≥99.5%,
   26/30 ≥99.0%. vs gate3 on 15 comparable seeds: net −77,171 cells, 12
   improved, 2 tiny regressions (60606 +173, 70707 +972). Bottom four:
   55555 97.73 (iceberg parked), 777 98.71, 456 98.92, 424242 98.94. All
   four are the known border/origin-order + tree-stream cascade; no new
   independent writer. Tooling note: /tmp tmpfs hit 100% during the run
   (7G parity-cache + stale ndec dumps) — freed by deleting stale cache
   fingerprints; keep /tmp clear before long multi-seed runs.
   Tests green; all pushed (origin/main clean).
3. **STEP-7 UNION FIX LANDED (4 Sep s26, commit 5b03feb)**:
   apply_step_origin early-returned when features_at_step(primary_biome,
   gen_step) was empty (plains step 7 = []) BEFORE building the 3x3
   biome-union feature list — silently skipping the whole decoration step
   for origins whose neighbours included cave biomes (dripstone_caves step
   7 = dripstone_cluster + pointed_dripstone; ~951k dripstone cells across
   12 seeds). Fix: union computed first; early-return only when both union
   and primary list are empty. Measured: 70707 371,432→289,715
   (**99.4376%**, −81,717); 777 674,071→671,026 (98.7047%); 12345
   448,459→448,365 (99.1362%); 424242 562,139→562,057 (98.9109%). All
   improved. GATE3 (5 Sep s27): all 27 remaining seeds re-run — 26/27
   improved, net −571,589 cells; mean 99.3503%, 11/27 ≥99.5%. CLOSED.
1b. **OBJECTIVE — seed 456 (98.8722% / 584,256; the only non-55555 seed
   below 99.0)**. Writers ledger (5 Sep s27): terrain 277k / tree 195k /
   simple_block 65k / vegetation_patch 12k. Jungle-signature cells 267k
   (86% border); biomes match 100% at the worst chunks. Iteration 1:
   ported mega_jungle trunk+foliage, bush foliage, leave_vine+cocoa
   decorators — REVERTED, 456 regressed to 98.7551% (+60,681): the extra
   RNG draws desync the shared selector stream (vanilla mega picks consume
   branch draws neutron previously skipped; with the port the whole
   trees_jungle stream shifts and canopies land where the ref has none).
   The selector itself (chances 0.1/0.5/0.333/0.0125 + default) matches
   vanilla RandomSelectorFeature draw-for-draw. Iteration 2: frozen
   WORLD_SURFACE_WG/OCEAN_FLOOR_WG heightmaps (vanilla freezes Usage.WORLDGEN
   maps at surface; ProtoChunk tracks only FINAL maps during features,
   ChunkStatus.java:17-28 + ProtoChunk.java:147-169) — REVERTED, 456
   regressed to 98.7296% (+73,871): the frozen read is measurably wrong too,
   so vanilla WG heightmaps must reflect something between live and frozen
   (candidate: the WG maps DO get updated by setBlock because
   getOrCreateHeightmapUnprimed lazily primes them at first setBlock, or
   WorldGenRegion.getHeight re-primes). 2 investigation+2 fix iterations
   spent on 456; per the 5-cap, NEXT: park 456 with both negative results
   recorded; move to the 10101/789/50000 cluster ledger (99.01–99.08) for
   an independent writer before returning with a better WG-heightmap model.
   LEDGERS DONE (5 Sep s27): 10101 #1 writer = ancient_city (244k of 512k —
   structure not ported; PARKED like 55555, too large). 789 #1 = mangrove
   trees 138k (upwards_branching_trunk + random_spread_foliage +
   mangrove_root_placer — all Unknown in neutron → trees place nothing).
   Iteration 3: ported mangrove trunk/foliage/roots — REVERTED, 789
   regressed to 98.9862% (+33,673). SAME failure mode as 456's mega-jungle
   port: any tree-port that starts consuming selector-stream draws at
   origins where neutron previously no-oped REGRESSES, proving the shared
   decoration RNG stream is ALREADY desynced before tree selection (the
   no-op was accidentally absorbing the desync). Trees are downstream of
   the real divergence — root cause sits EARLIER in the per-origin step
   chain (steps 1..6: placement-modifier draws). STREAM DUMP DONE (5 Sep
   s28, oracle789b.ndec + pfd-mang.out + neu-789-trace.log, origin (-2,0)):
   the stream matches THROUGH tree selection — vanilla n=5 mangrove at
   (-23,9,y74): selroll 0.6160519<0.85 → tall_mangrove_checked, height
   4+0+4=8 — neutron draw 6: identical roll, identical nextInt(2)=0,
   nextInt(10)=4, ACCEPT at the same cell. The divergence starts INSIDE the
   accepted tree: vanilla then consumes the full tree's draws (mangrove
   roots simulate, trunk branches, 70 foliage attempts, decorators → 827
   writes) while neutron's Unknown-trunk no-op consumes ~0, so the NEXT
   attempt's in_square draws diverge (van n=6 = (10,2), neu = (9,8)). The
   earlier mangrove port REGRESSED because its INTERNAL draw order didn't
   match vanilla's (root placer/above-root/foliage sequence), not because
   selection desyncs. ITERATION 4 (5 Sep s28): re-landed the port with the
   EXACT vanilla order — two real bugs found and fixed via raw-stream diff
   (pfd-mang47.out STREAM gif=47 vs NEUTRON_RNG_TRACE): (a) doPlace draws
   getTreeHeight BEFORE rootPlacer.getTrunkOrigin (TreeFeature.java:65-69 —
   my first port had it reversed); (b) BlockPos.distManhattan INCLUDES the Y
   term (my width omitted it). After fixes the streams match 15+ draws
   (selector 0.6160519 → height 0,4 → offset 0 → root-skew dice identical).
   Full 789 parity: 98.9869% (+33,303) — STILL REGRESSES. CORRECTION (5 Sep
   s28, surface_height_dump): the mud FLOOR heights MATCH vanilla (floors
   equal on nearly all columns; the surface diffs in raw dumps are tree
   canopies, expected). The terrain-gate hypothesis is WITHDRAWN. The draw
   divergence at index 15 (vanilla bool vs neutron float) is a root-walk
   POSITION difference: vanilla simulated a position where canPlaceRoot
   failed (no bool drawn), neutron walked elsewhere. Resolving it needs
   vanilla-side root-position logging — ITERATION 5 (5 Sep s28): fixed bug
   #3 (root direction order: vanilla Plane.HORIZONTAL = N,E,S,W; mine was
   W,E,N,S — found via ProbeMangroveRootTrace replay + python walk diff).
   Stream alignment improved 15→62 draws, then diverges on walk LENGTH:
   vanilla's dir-1 walk ends at ~62 dice (canPlaceRoot=false at a solid
   cell), neutron's continues (cell replaceable in neutron's terrain). The
   two terrains differ in stone/dirt boundary micro-cells in the swamp;
   each such cell flips canPlaceRoot and desyncs the rest of the origin's
   stream. Full 789: 98.9972% (+27,956) — still negative. BAIL OUT (5-cap):
   port REVERTED (code in session history; bugs #1 height-before-offset,
   #2 distManhattan Y, #3 dir order are all real fixes to re-apply once
   terrain micro-parity lands). OBJECTIVE MOVES TO PHASE 1: quantify and
   fix the stone/dirt-boundary micro-diffs in mangrove_swamp (789) and
   jungle floor (456) — doFill/surface level, BEFORE any tree port can
   net-positive. ProbeMangroveRootTrace committed (tools/); AUTHORITATIVE
   vanilla root walk captured (evidence/stream789/vanilla-root-walk-
   authoritative.txt, ROOTWALK=1 in-scene): N 6 roots, E 3, S 3, W 8, first
   candidate (-23,77,7) via else-bool(true), all canPlace results logged.
   ITERATION 6 DONE (5 Sep s28): re-applied port + instrumentation, walked
   both sides position-for-position. FIRST DIVERGENCE: canPlaceRoot at
   (-20,75,9) — neutron canPlace=true (mud). Vanilla-side comparison from
   the post-tree replay was INCONCLUSIVE: that trace is a POST-TREE replay
   (hook runs after cf.place()), so its canPlace results include the tree's
   own writes (below-trunk dirt at 75) — NOT the live walk semantics. The
   live-vs-replay mismatch makes further vanilla-side
   diffing impossible without instrumenting inside MangroveRootPlacer
   (javaagent/reflection hook — heavy).
   port draw-exact through 62 dice; 3 measurements all negative; port
   REVERTED; mangrove objective PARKED until either (a) a live-walk
   vanilla tracer exists, or (b) a different angle (e.g. count parity of
   mangrove_roots per chunk) yields a cheap check. Surface-rule "dirt vs
   mud" claim WITHDRAWN (replay artifact). CROSS-SCENE WALK PROOF (5 Sep
   s28): replaying vanilla dice on the NEUTRON-scene NDEC reproduces
   neutron's walk exactly ((-20,76,9)T -> (-20,75,9)F), and on the
   VANILLA-scene reproduces vanilla's — the port + dir order are fully
   correct; the entire residual divergence is the underlying terrain at
   cells the root walk touches. QUANTIFIED (5 Sep s28, z=9 slice
   x=-40..-20): 5/20 columns differ, ALL the same pattern — the TOP of the
   mud band: vanilla = dirt,dirt,grass_block; neutron = mud,mud,mud
   (e.g. x=-27: 68-70). IT 7-8: vanilla surface rule decoded (mangrove mud
   is UNCONDITIONAL in the biome under-chain; dirt@75 in the old trace was
   the tree's own below_trunk write — replay artifact). Live-walk tracer
   built (two-pass NDEC restore + dice replay): origin (-2,0) tree matches
   vanilla 31/31 root calls. But chunk (2,-1): vanilla places a mangrove
   tree, neutron never fires it (0 root calls) — stream already desynced
   upstream. Full 789 with verified port: 98.9972% (+27,956) still
   negative. CLIMATE PARAM MYTH BUSTED (ProbeClimateAt, quart coords): the
   earlier all-6-params reading was an ARTIFACT (sampler.sample takes QUART
   coords, not block). With quart coords vanilla == neutron EXACTLY on all
   six params at every probed point. LIVE TREE DIFF (authoritative write
   log vs neutron final): origin (-2,0) tree = 24/26 root positions EXACT
   (S-direction 2-cell diff: (-23,76,10) missing, (-23,75,11) mud-vs-muddy).
   The tree port is essentially correct; the 789 regression (+27,956 with
   port) comes from OTHER origins whose upstream streams desync. PARKED.
   -12025 vs neutron +2974 — DEPTH DIFFERS EVERYWHERE); temp matches at
   origin but differs at (100,70,100) and (-200,70,-200). Summary: depth
   is globally wrong (sign+scale), temperature/vegetation diverge only
   far from origin (shifted-noise jitter or noise-impl precision at
   boundaries — matches the boundary-only biome flips seen in parity).
   depth df = add(y_clamped_gradient(1.5→-1.5 over -64..320), overworld/
   offset spline chain). Neutron gradient impl verified correct; the
   delta pattern (van - neu = 1.50@64, 1.78@75, 1.10@46 on one column)
   is NOT constant — the overworld/offset spline chain evaluates
   differently. NEXT (iteration 9): dump overworld/offset spline inputs
   (continents spline) both sides at (-20,75,9); the offset chain is the
   depth fix; the shifted-noise jitter is the temp/humid fix.
   climate_at example). Mangrove port stays PARKED until climate parity
   lands (re-apply is mechanical — bugs #1-#3 + dir order documented;
   VANILLA_NDEC_OUT export hook fixed, LE writer verified).
   WG-FROZEN FIX LANDED (6 Sep s29, 36427c5): WORLD_SURFACE_WG reads the
   frozen post-surface snapshot (RegionBuf per-chunk heightmaps) instead
   of the live buffer — vanilla ChunkStatus.SURFACE re-primes WG maps
   then freezes them (final maps stay live-tracked). Ratchet: 456
   −23,548 (→98.9177%), 789 −1,746 (→99.0546%, crosses 99.0), 424242
   −17,279 (→98.9444%), 70707 +972 (→99.4357%, small mangrove-adjacent
   exception). Full ratchet (10 seeds): 50000 −4,200 (99.0903), 12345
   −790 (99.1377), 777 −3,870 (98.7122), 10101 −2,662 (99.0150),
   22222 −1,952 (99.1895) — net −54,203 across 10 seeds; only 70707
   +972 against (0.002pp). Remaining 20 gate seeds queued; tests green.
   50000 (99.0821%/474,604) — spruce trees 138k + dripstone_caves 138k,
   BOTH 86% border; core-only 19.5k/18k. Same signature as 456/789: the
   remaining gap across every 99.0-99.4 seed is the border origin-order
   cascade plus tree/stream divergence — NO new independent writer. The
   cheap per-seed ledger route is exhausted; the remaining ~5M gate gap
   requires either the live-walk tracer (javaagent) or a structural fix
   to the origin-order model (95.85% fit ceiling).
4. **HEIGHTMAP FIX LANDED (3 Sep s24, commit 3d66868)**: vanilla
   buildSurface's `height` = WORLD_SURFACE_WG+1 INCLUDES fluids
   (SurfaceSystem.java:112,119); neutron passed a fluid-EXCLUSIVE heightmap,
   so the surface y-loop started below the water column, water_height stayed
   MIN at deep ocean floors, the Water(-6) condition passed (MIN = exposed)
   and sediment dirt/sand overwrote stone floors. Fix: apply_surface_rules
   recomputes the fluid-inclusive top per column before the y-loop.
   Measured: 424242 568,109→562,139 (98.9108%); 12345 756,361→448,459
   (**99.1360%**); 777 717,926→674,071 (98.6989%); 40000 565,427→130,108
   (**99.7489%**). Aggregated `stone->dirt` across 30 seeds: 1.20M cells.
   NEXT: re-run the full 30-seed gate with this fix (expect ~1M cell drop);
   iceberg chain still parked.
5. **30-SEED VALIDATION COMPLETE (3 Sep s21)**: 27 new refs generated with
   29/30 seeds in 98.54–99.76% (mean 99.10); 17 seeds ≥99.0; best 33333
   99.7587/124,776. Outlier: **55555 = 96.9196/1,589,788** — deep_frozen_ocean
   packed-ice bergs (727k cells = 51% of its gap; zero reverse cells) plus
   frozen floor surface rules (stone→dirt 201k, gravel→dirt/sand).
   Seed 123 also carries 143k packed-ice gap. ROOT CAUSE: vanilla
   `SurfaceSystem.frozenOceanExtension` (SurfaceSystem.java:235-284) —
   iceberg_surface/iceberg_pillar(x*1.28)/iceberg_pillar_roof(x*1.17) noises
   build giant snow/packed-ice columns — was never ported (noises ARE in
   datapack_data.rs:79-82).
6. **frozenOceanExtension objective Bailed OUT (3 Sep s23, 5-iteration
   cap)**: port tested: 55555 **−423,814** (96.92→97.74%), 424242
   bit-identical, 123 **+45,387** → reverted per ratchet rule. Live-server
   experiment (probe-123 world, real 26.2 jar, forceload) proved the ref
   IS real vanilla: berg columns byte-identical. Column-level ground
   truth: extension fills water band [sea−top−7 .. sea] with ~15% skips,
   top = min(berg²·1.2, ceil(roof·40)+14)+sea; the 123 regression came
   from neutron fills landing where the ref has feature cut-outs and
   inter-column variation the port cannot see without the full
   IcebergFeature interplay (feature bergs + cut-outs carve the
   extension ice; my earlier "ref lacks ice" and "ref stone-to-63"
   readings were ledger/column-sampling misreads — ref terrain matches
   neutron at non-berg floors). The remaining lever is a full
   IcebergFeature + cut-out + extension joint implementation — parked:
   too large for the iteration budget; revisit if the gate moves above
   99.5 on the other 29 seeds. Port stays reverted; probe evidence
   committed (ProbeIcebergNoise/Msl/BiomeAtXY).
7. Origin order model CLOSED (2 Sep s19) — see below. 30-seed gate:
   ≥99.5 NOT met on all seeds (floor 98.54 outside 55555); gate accepted
   at established per-seed baselines until the iceberg chain lands.
8. place_on_ground vine acceptance TESTED and REVERTED (2 Sep s19):
   vanilla PlaceOnGroundDecorator.java:80 accepts above ∈ {air, VINE};
   neutron only air. Enabling vine acceptance regressed 424242 to
   568,965 (+856) — neutron's vine positions diverge from vanilla's
   (origin-order cascade), so extra accepts write leaf_litter where
   vanilla has air. Reverted; decision recorded in
9. **Fresh writers ledger (2 Sep s19, partial ~322k rows before
   stop)**: top writers unchanged — terrain-missing (dark_oak_leaves
   19.5k, dark_oak_log 7.6k, pale_oak_leaves 6.4k, oak_leaves 5.6k,
   leaf_litter 4.6k), tree-extra, vegetation_patch, simple_block,
   block_column. ALL dominated by the border/origin-order cascade;
   simple_block confusions (short_grass↔moss_carpet, water→short_grass
   in lush pools) trace to the same chain. No new independent writer.
10. Ruined portal loot tables (out of metric). AGENTS.md ref paths for
   12345/777 DO have `world/` prefix (stale doc).
11. `cargo test --workspace` before any push.

## Perf / Environment (this box)

8 cores; meter default leaves 2 free (`PARITY_WORKERS`). Chunk gen ~9.5
s cold, ~5 s warm. Rust 1.98 · Temurin 25 · vineflower 1.12. Probe
rebuild recipe in tools/worldgen-probe/src. Playbook: docs/PARITY.md.