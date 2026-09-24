shader_type spatial;

uniform sampler2D image : filter_nearest;

void vertex() {
	UV = (INSTANCE_CUSTOM.xy + UV) * 0.03125;
}

void fragment() {
	// Called for every pixel the material is visible on.
	ALBEDO = texture(image, UV).rgb;
}

//void light() {
//	// Called for every pixel for every light affecting the material.
//	// Uncomment to replace the default light processing function with this one.
//}
