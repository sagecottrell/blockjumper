shader_type spatial;

uniform sampler2D albedo_texture : source_color, filter_nearest;
uniform vec4 input_color1 : source_color;
uniform vec4 output_color1 : source_color;

bool colorMatch(vec4 c1, vec4 c2, float maxDiff) {
    return (
        abs(c1.r - c2.r) < maxDiff
        && abs(c1.g - c2.g) < maxDiff
        && abs(c1.b - c2.b) < maxDiff
    );
}

void fragment() {
	vec4 current_color = texture(albedo_texture, UV);
	if (colorMatch(current_color, input_color1, 0.01))
		ALBEDO = output_color1.rgb;
	else
		ALPHA = 0.0;
}
