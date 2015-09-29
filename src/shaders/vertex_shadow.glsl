#version 330

in vec3 position;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
	gl_Position = perspective * view * model * vec4(position, 1.0);
}