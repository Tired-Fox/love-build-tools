-- lovr-phong
-- A GLSL lighting tutorial for LOVR
-- Written by Ben Ferguson, 2020
-- CC0

function lovr.load()
  -- Initialize our model
  model = lovr.graphics.newModel('zxx-1.glb')

  -- set up shader
  shader = lovr.graphics.newShader([[
    vec4 lovrmain() {
      return Projection * View * Transform * VertexPosition;
    }
  ]], [[
    Constants {
      vec4 ambience;
      vec4 lightColor;
      vec3 lightPos;
      float specularStrength;
      int metallic;
    };

    vec4 lovrmain() {
      // Diffuse
      vec3 norm = normalize(Normal);
      vec3 lightDir = normalize(lightPos - PositionWorld);
      float diff = max(dot(norm, lightDir), 0.0);
      vec4 diffuse = diff * lightColor;

      // Specular
      vec3 viewDir = normalize(CameraPositionWorld - PositionWorld);
      vec3 reflectDir = reflect(-lightDir, norm);
      float spec = pow(max(dot(viewDir, reflectDir), 0.0), metallic);
      vec4 specular = specularStrength * spec * lightColor;

      vec4 baseColor = Color * getPixel(ColorTexture, UV);

      return baseColor * (ambience + diffuse + specular);
    }
  ]])
end

function lovr.draw(pass)
  pass:setShader(shader)

  local time = lovr.timer.getTime()
  local lightPos = vec3(math.sin(time)*2, -1.0, 3.0)

  -- Set shader values
  pass:send('ambience', {0.05, 0.05, 0.05, 1.0})
  pass:send('lightColor', {1.0, 1.0, 1.0, 1.0})
  pass:send('lightPos', lightPos)
  pass:send('specularStrength', 0.5)
  pass:send('metallic', 32.0)

  pass:draw(model, 0, -2, -3, 1, 0)

  pass:setShader() -- Reset to default/unlit
  pass:text('hello world', 0, 1.7, -3, .5)
  pass:sphere(lightPos, -1, -3, 0.1) -- Represents light
end
