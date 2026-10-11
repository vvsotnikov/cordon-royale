// Classify the shader, not words in the texture filename. X-Ray's transparent
// world shaders are also used for wire/net cutouts, not just foliage.
export function usesAlphaCutout(shader){
 const name=shader.split('/')[0].replaceAll('\\','/').toLowerCase();
 return /aref|tree|flora|fence|leaves/.test(name)||/(^|\/)def_trans(_v)?$/.test(name);
}
