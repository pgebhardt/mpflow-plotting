#version 330

layout(location=1) out float depth;

void main() {
	depth = gl_FragCoord.z;
}