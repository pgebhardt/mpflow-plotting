#version 330

in vec3 v_normal;
in vec3 v_position;
in float v_value;
in vec4 shadow_position;

out vec4 frag_color;

uniform vec3 light_pos;
uniform mat4 view;
uniform sampler2D shadow_map;

const vec3 specular_color = vec3(1.0, 241.0 / 255.0, 224.0 / 255.0);

vec3 jet_color(float value) {
	// correct value space
	float normalized_value = 0.5 * value + 0.5;
	
	// calculate colors
	float red = min(max(-4.0 * abs(normalized_value - 0.75) + 1.5, 0.0), 1.0);
    float green = min(max(-4.0 * abs(normalized_value - 0.5) + 1.5, 0.0), 1.0);
    float blue = min(max(-4.0 * abs(normalized_value - 0.25) + 1.5, 0.0), 1.0);
	
	return vec3(red, green, blue);
}

void main() {
	// calculate color;
	vec3 v_color = jet_color(v_value);
		
	// calculate direction to light source
	vec3 light_dir = normalize(light_pos - v_position);

	// calculate diffuse color component
	float diffuse = max(dot(light_dir, normalize(v_normal)), 0.0);

	// calculate specular color components
	vec3 camera_dir = normalize(-v_position);
	vec3 half_dir = normalize(light_dir + camera_dir);
	float specular = diffuse >= 0.0 ? pow(max(dot(half_dir, normalize(v_normal)), 0.0), 16.0) : 0.0;
	
	// get visibility from shadow map
	float bias = 1e-4 * clamp(tan(acos(clamp(dot(normalize(v_normal), light_dir), 0.0, 1.0))), 0.0, 4.0);
	if (texture(shadow_map, shadow_position.xy / shadow_position.w).r < (shadow_position.z - bias) / shadow_position.w) {
		diffuse *= 0.5;
		specular = 0.0;
	}
	
	// calculate fragment color
	frag_color = pow(vec4((0.3 + 0.7 * diffuse) * v_color + 0.2 * specular * specular_color, 1.0), vec4(2.2, 2.2, 2.2, 1.0));
}
