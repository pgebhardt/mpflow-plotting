#version 330

in vec3 v_normal;
in vec3 v_position;
in vec3 camera_dir;
in vec4 shadow_position;

out vec4 frag_color;

uniform vec3 light_pos;
uniform vec3 diffuse_color;
uniform mat4 view;
uniform sampler2D shadow_map;

const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
	// calculate direction to light source
	vec3 light_dir = normalize(light_pos - v_position);

	// calculate diffuse and specular color components
	vec3 half_dir = normalize(light_dir + camera_dir);
	float diffuse = max(dot(light_dir, normalize(v_normal)), 0.0);
	float specular = diffuse >= 0.0 ? pow(max(dot(half_dir, normalize(v_normal)), 0.0), 16.0) : 0.0;
	
	// get visibility from shadow map
	float bias = clamp(0.005 * tan(acos(clamp(dot(normalize(v_normal), light_dir), 0.0, 1.0))), 0.0, 0.01);
	if (texture(shadow_map, shadow_position.xy / shadow_position.w).r < (shadow_position.z - bias) / shadow_position.w) {
		diffuse *= 0.5;
		specular = 0.0;
	}
	
	frag_color = vec4(0.2 * diffuse_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}