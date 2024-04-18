
/**
 * Take an array of bytes (Uint8array) and convert it to a Float32Array for processing
 * @param data The binary data
 */
export function convertUInt8ToFloat32(buffer: ArrayBuffer) {
    // let uint8Array = new Uint8Array(buffer);

    var float32Array = new Float32Array(buffer); // Assuming each sample is 2 bytes (16 bits)

    // [28, 48,     18, 40,    28, 48,         18, 40]
    // [smpl1 L     smpl1 L,   smpl1 R         smpl1 R]
    // To:
    // [-0.4, 0.3, -0.53, 0.22] // interleaved samples
    // [s1L, s1R,   s2L,   s2R]
    let floatIdx = 0;
    for (var i = 0; i < float32Array.length; i += 1) {
        // Combine two 8-bit values into a 16-bit signed integer
        // var uint16Value = (uint8Array[i + 1] << 8) + uint8Array[i];
        // var normalized = uint16Value / 0x8000;
        // if (normalized < 0x8000) {
        // Convert to a float value in the range of -1 to 1
    }
    // float32Array[i] = float32Array[i] / 0x8000;

    floatIdx++;

    return float32Array;
}