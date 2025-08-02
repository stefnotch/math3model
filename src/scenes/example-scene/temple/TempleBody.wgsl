fn Cube(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
    var p = vec3f(input2.x * TWO_PI, input2.y, 0.0);
    let adjusted_angle = p.x + TWO_PI / 8.0;
    var s = sin(adjusted_angle);
    var c = cos(adjusted_angle);
    var y0 = (step(0.001,fract(p.y)));
    var a = abs(s) + abs(c);

    var box = vec3f(
        (s + c) / a * y0,
        input2.y,
        (s - c) / a * y0
    );
    return box;
}

fn sampleObject(input2: vec2f) -> vec3f {
	var ref_58504 = Cube(input2);
	var ref_3498a = mat3x3(vec3f(3.5,0.0,0.0), vec3f(0.0,10,0.0), vec3f(0.0,0.0,0.6)) * ref_58504;
	var ref_578c8 = 1.00000000000000000000;
	var ref_e8658 = f32(8);
	var ref_c8440 = ref_e8658 * 1.19999999999999995559;
	var ref_d944a = vec3f(ref_578c8, ref_578c8, ref_c8440);
	var ref_0e240 = ref_d944a * ref_3498a;
	var ref_985dd = ref_d944a * 0.50000000000000000000;
	var ref_99140 = ref_0e240 + ref_985dd;
	var ref_1a80b_1 = ref_99140[0];
	var ref_1a80b_2 = ref_99140[1];
	var ref_1a80b_3 = ref_99140[2];
	var ref_a3d2f = ref_e8658 / 2.00000000000000000000;
	var ref_3354f = ref_1a80b_3 - ref_a3d2f;
	var ref_7d947 = vec3f(ref_1a80b_1, ref_1a80b_2, ref_3354f);
	return ref_7d947;

}

fn getColor(input: vec2f, base_color: vec3f) -> vec3f {
    return base_color;
}