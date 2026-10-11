import assert from 'node:assert/strict';
import {usesAlphaCutout} from '../tools/world-materials.mjs';
for(const shader of ['def_shaders\\def_trans/prop\\prop_barbwire3,lmap#1','def_shaders\\def_trans_v/prop\\prop_mask_setka','def_shaders\\def_aref/prop\\glass','flora\\tree/lfo\\leaves'])assert(usesAlphaCutout(shader),shader);
for(const shader of ['default/wood\\wood_fence_01','default/lfo\\lftrees_bark_a_02','default/crete\\crete_fence_02','def_shaders\\def_vertex/prop\\prop_transformer_dw'])assert(!usesAlphaCutout(shader),shader);
console.log('PASS wire/net alpha shaders and opaque fence/bark materials');
