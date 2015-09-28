#version 330

in vec3 v_normal;
in vec3 v_position;

out vec4 color;

uniform vec3 light_pos;

const vec3 ambient_color = vec3(0.2, 0.0, 0.0);
const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
	// calculate direction to light source
	vec3 light_dir = normalize(light_pos - v_position);

	// calculate diffuse color part
	float diffuse = max(dot(light_dir, normalize(v_normal)), 0.0);

	vec3 camera_dir = normalize(-v_position);
	vec3 half_dir = normalize(light_dir + camera_dir);
	float specular = diffuse >= 0.0 ? pow(max(dot(half_dir, normalize(v_normal)), 0.0), 16.0) : 0.0;

	color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}