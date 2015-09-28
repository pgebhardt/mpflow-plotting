#version 330

in vec3 v_position;

layout(location=1) out float depth;

void main() {
	depth = v_position.z;
}