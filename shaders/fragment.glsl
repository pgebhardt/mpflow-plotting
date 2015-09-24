#version 150

in vec3 v_normal;
in vec3 v_position;

out vec4 color;

uniform vec3 u_light;
uniform mat4 view;

const vec3 ambient_color = vec3(0.2, 0.0, 0.0);
const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
const vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
	vec3 light_position = (view * vec4(u_light, 1.0)).xyz;
	float diffuse = max(dot(normalize(v_normal), normalize(light_position)), 0.0);

	vec3 camera_dir = normalize(-v_position);
	vec3 half_direction = normalize(normalize(light_position) + camera_dir);
	float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
	
	color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}