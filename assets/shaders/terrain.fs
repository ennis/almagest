#version 440

// uniform buffer bindings:
// 0: scene and pass global (includes lights)
// 1: per-material data
// 2: per-object data

layout (std140, binding = 0) uniform SceneData {
	mat4 viewMatrix;
	mat4 projMatrix;
	mat4 viewProjMatrix;	// = projMatrix*viewMatrix
	vec4 lightDir;
	vec4 wEye;	// in world space
	vec2 viewportSize;	// taille de la fenêtre
	vec3 wLightPos;
	vec3 lightColor;
	float lightIntensity;
};

in vec2 tc;
in vec3 wPos;
in vec3 wN;
in float relHeight;
out vec4 color;

void main() {
	color = vec4(relHeight, relHeight, relHeight, 0.0);
}
