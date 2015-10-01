#version 330

in vec3 position;
in vec3 normal;
in float value;

out vec3 v_normal;
out vec3 v_position;
out float v_value;
out vec4 shadow_position;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

uniform mat4 shadow_perspective;
uniform mat4 shadow_view;
const mat4 shadow_bias = mat4(
	vec4(0.5, 0.0, 0.0, 0.0),
	vec4(0.0, 0.5, 0.0, 0.0),
	vec4(0.0, 0.0, 0.5, 0.0),
	vec4(0.5, 0.5, 0.5, 1.0));

void main() {
	v_value = value;
	
	// position of vertex in clip space
	gl_Position = perspective * view * model * vec4(position, 1.0);

	// normal of vertex in camera space
	v_normal = transpose(inverse(mat3(view * model))) * normal;
	
	// position of vertex in camera space
	v_position = (view * model * vec4(position, 1.0)).xyz;

	// direction of vertex to camera in camera space

	// position of vertex in shadow space
	shadow_position = shadow_bias * shadow_perspective * shadow_view * model * vec4(position, 1.0);
}
