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

// texture bindings:
// 0-3: pass textures (shadow maps, etc.)
// 4-8: per-material textures
// 8-?: per-object textures

layout (binding=0) uniform sampler2D mainTex;
layout (binding=1) uniform sampler2D shadowTex;

layout (std140, binding = 2) uniform LightData {
	mat4 shadowDepthMatrix;
};

in vec2 tc;
in vec4 shadowCoords;
in vec3 wPos;
in vec3 wN;
out vec4 color;

vec4 PhongIllum(
	vec4 albedo,
	vec3 normal,
	vec3 lightDir,
	vec3 position,
	float ka,
	float ks,
	float kd,
	vec3 lightIntensity,
	float eta,
	float shininess)
{
	vec4 Ln = normalize(vec4(lightDir, 0.0)),
         Nn = normalize(vec4(normal, 0.0)),
         Vn = normalize(wEye - vec4(position, 1.0f));
    vec4 H = normalize(Ln + Vn);
    vec4 Li = vec4(lightIntensity, 1.0);
    // Ambient
    vec4 ambient = ka * Li * albedo;
    // Diffuse
    vec4 diffuse = kd * max(dot(Nn, Ln), 0.0) * albedo;
    // Specular
    //vec4 Rn = reflect(-Ln, Nn);
    //vec4 specular = ks * albedo * pow(max(dot(Rn, Vn), 0.0), shininess) * Li;
    //specular *= fresnel(eta, dot(H, Vn));
	return vec4((ambient + diffuse).xyz, 1.0);
	// TEST
	//return vec4(position, 1.0f);
}

void main() {
	//color = vec4(tc, 0.0, 0.0);
	float visibility = (texture(shadowTex, shadowCoords.xy).z < shadowCoords.z - 0.005) ? 1.0 : 0.2;
	color = visibility * PhongIllum(
		texture(mainTex, tc),
		wN,
		wPos - wLightPos,
		wPos,
		0.2, 0.0, 0.8, lightIntensity * lightColor, 0.0, 0.0);
	//color = vec4(texture(shadowTex, shadowCoords.xy).r);
	color = vec4(visibility);
}
