import package::uniforms_0::{time, screen, mouse, extra, instance_id};

fn sampleObject(input2: vec2f) -> vec3f {
	var ref_0c994_1 = input2[0];
	var ref_0c994_2 = input2[1];
	var ref_f48 = 10.00000000000000000000;
	var ref_dc7f0 = ref_0c994_2 * ref_f48;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_11910 = TWO_PI * ref_0c994_1;
	var ref_88b27 = cos(1 * ref_11910 + 0);
	var ref_35579 = ref_88b27 * 0.80000000000000004441;
	var ref_cddcc = sin(1 * ref_11910 + 0);
	var ref_ae854 = ref_cddcc * 0.80000000000000004441;
	var ref_c1dcd = vec3f(ref_35579, ref_dc7f0, ref_ae854);
	var ref_2aaca = vec3f(ref_35579, 0, ref_ae854);
	var ref_ea4c0 = ref_11910 * 10.00000000000000000000;
	var ref_a7101 = sin(1 * ref_ea4c0 + 0);
	var ref_1c4ba = abs(ref_a7101);
	var ref_69e24 = ref_1c4ba * -0.05000000000000000278;
	var ref_e78a8 = ref_2aaca * ref_69e24;
	var ref_e1bd6 = ref_e78a8 + ref_c1dcd;
	var ref_23d50 = sin(1 * ref_0c994_2 + 2.5);
	var ref_94c5f = ref_23d50 * 0.40000000000000002220;
	var ref_481c0 = ref_2aaca * ref_94c5f;
	var ref_a26c1 = ref_e1bd6 + ref_481c0;
	var ref_07213 = 0.00000000000000000000;
	var instanceId = f32(instance_id);
	var ref_6f597 = instanceId % 2.00000000000000000000;
	var ref_b6cd7 = ref_6f597 * 3.00000000000000000000;
	var ref_9d3cc = instanceId / 2.00000000000000000000;
	var ref_3be67 = floor(ref_9d3cc);
	var ref_47062 = ref_3be67 * 2.00000000000000000000;
	var ref_2df4e = ref_47062 * 1.50000000000000000000;
	var ref_62e9e = f32(8);
	var ref_2ae8d = ref_62e9e / 2.00000000000000000000;
	var ref_b590a = ref_2df4e - ref_2ae8d;
	var ref_81f81 = vec3f(ref_b6cd7, ref_07213, ref_b590a);
	var ref_5517c = ref_a26c1 + ref_81f81;
	return ref_5517c;

}


fn getColor(input: vec2f, base_color: vec3f) -> vec3f {
    return base_color;
}