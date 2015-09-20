#ifndef SHADOW_MAP
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
#endif

#ifdef SHADOW_MAP
layout (std140, binding = 0) uniform LightData {
	mat4 lightTransformMatrix;
};
#endif

#ifdef SHADOWS_SIMPLE
layout (std140, binding = 2) uniform LightData {
	mat4 shadowDepthMatrix;
};
#endif

#ifdef _VERTEX_
vec4 wPosToClipSpace(vec4 pos)
{
	#ifndef SHADOW_MAP
	return projMatrix * viewMatrix * pos;
	#endif
	#ifdef SHADOW_MAP
	return lightTransformMatrix * pos;
	#endif
}
#endif

// Do not compile illum functions when rendering a shadow map
#ifndef SHADOW_MAP
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
#endif

