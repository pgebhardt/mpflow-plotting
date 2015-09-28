#version 330

in vec3 position;

out vec3 v_position;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
	gl_Position = perspective * model * view * vec4(position, 1.0);
	v_position = gl_Position.xyz / gl_Position.w;
}