#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

// Helper function to load template files
function loadTemplate(templateName) {
  const templatePath = path.join(__dirname, "templates", templateName);
  return fs.readFileSync(templatePath, "utf8");
}

// Simple TypeScript client generator that creates Zod schemas and fetch wrappers
// This is a lightweight alternative to heavy OpenAPI generators

const API_SPEC = {
  baseUrl: "http://localhost:8080",
  endpoints: {
    registerStart: {
      method: "POST",
      path: "/register_start/{username}",
      pathParams: ["username"],
      queryParams: {
        invite_code: "string?",
      },
      requestSchema: "void",
      responseSchema: {
        publicKey: {
          challenge: "string",
          rp: {
            id: "string",
            name: "string",
          },
          user: {
            id: "string",
            name: "string",
            displayName: "string",
          },
          pubKeyCredParams: "array",
          timeout: "number",
          excludeCredentials: "array",
          authenticatorSelection: {
            residentKey: "string",
            requireResidentKey: "boolean",
            userVerification: "string",
          },
          attestation: "string",
          extensions: "object?",
        },
      },
    },
    registerFinish: {
      method: "POST",
      path: "/register_finish",
      requestSchema: {
        id: "string",
        rawId: "string",
        response: {
          attestationObject: "string",
          clientDataJSON: "string",
        },
        type: "string",
      },
      responseSchema: "void",
    },
    loginStart: {
      method: "POST",
      path: "/login_start/{username}",
      pathParams: ["username"],
      requestSchema: "void",
      responseSchema: {
        challenge: "string",
        timeout: "number",
        rp_id: "string",
        allow_credentials: "array",
        user_verification: "string",
      },
    },
    loginFinish: {
      method: "POST",
      path: "/login_finish",
      requestSchema: {
        id: "string",
        rawId: "string",
        response: "object",
        type: "string",
      },
      responseSchema: "void",
    },
    logout: {
      method: "POST",
      path: "/logout",
      requestSchema: "void",
      responseSchema: "void",
    },
    health: {
      method: "GET",
      path: "/health",
      requestSchema: "void",
      responseSchema: "void",
    },
  },
};

function generateZodSchema(schema, name) {
  if (typeof schema === "string") {
    const isOptional = schema.endsWith("?");
    const baseSchema = isOptional ? schema.slice(0, -1) : schema;

    let zodType;
    switch (baseSchema) {
      case "string":
        zodType = "z.string()";
        break;
      case "number":
        zodType = "z.number()";
        break;
      case "boolean":
        zodType = "z.boolean()";
        break;
      case "array":
        zodType = "z.array(z.any())";
        break;
      case "object":
        zodType = "z.object({})";
        break;
      case "void":
        zodType = "z.void()";
        break;
      default:
        zodType = "z.any()";
        break;
    }

    return isOptional ? `${zodType}.optional()` : zodType;
  }

  if (Array.isArray(schema)) {
    return `z.array(${generateZodSchema(schema[0], name)})`;
  }

  if (typeof schema === "object") {
    const properties = Object.entries(schema)
      .map(
        ([key, value]) =>
          `    ${key}: ${generateZodSchema(value, `${name}_${key}`)}`,
      )
      .join(",\n");
    return `z.object({\n${properties},\n  })`;
  }

  return "z.any()";
}

function generateTypeFromSchema(schema, name) {
  if (typeof schema === "string") {
    const isOptional = schema.endsWith("?");
    const baseSchema = isOptional ? schema.slice(0, -1) : schema;

    let tsType;
    switch (baseSchema) {
      case "string":
        tsType = "string";
        break;
      case "number":
        tsType = "number";
        break;
      case "boolean":
        tsType = "boolean";
        break;
      case "array":
        tsType = "any[]";
        break;
      case "object":
        tsType = "object";
        break;
      case "void":
        tsType = "void";
        break;
      default:
        tsType = "any";
        break;
    }

    return isOptional ? `${tsType} | undefined` : tsType;
  }

  if (Array.isArray(schema)) {
    return `${generateTypeFromSchema(schema[0], name)}[]`;
  }

  if (typeof schema === "object") {
    const properties = Object.entries(schema)
      .map(
        ([key, value]) =>
          `  ${key}: ${generateTypeFromSchema(value, `${name}_${key}`)}`,
      )
      .join(";\n");
    return `{\n${properties};\n}`;
  }

  return "any";
}

function pascalCase(str) {
  return str.replace(/^[a-z]/, (char) => char.toUpperCase());
}

function generateApiClient() {
  const schemas = [];
  const types = [];
  const methods = [];

  // Generate schemas and types for each endpoint
  Object.entries(API_SPEC.endpoints).forEach(([endpointName, config]) => {
    const requestName = `${pascalCase(endpointName)}Request`;
    const responseName = `${pascalCase(endpointName)}Response`;

    // Request schema and type
    if (config.requestSchema !== "void") {
      schemas.push(
        `export const ${requestName}Schema = ${generateZodSchema(config.requestSchema, requestName)};`,
      );
      types.push(
        `export type ${requestName} = z.infer<typeof ${requestName}Schema>;`,
      );
    }

    // Response schema and type
    if (config.responseSchema !== "void") {
      schemas.push(
        `export const ${responseName}Schema = ${generateZodSchema(config.responseSchema, responseName)};`,
      );
      types.push(
        `export type ${responseName} = z.infer<typeof ${responseName}Schema>;`,
      );
    }

    // Generate method
    const hasRequest = config.requestSchema !== "void";
    const hasResponse = config.responseSchema !== "void";
    const hasPathParams = config.pathParams && config.pathParams.length > 0;
    const hasQueryParams =
      config.queryParams && Object.keys(config.queryParams).length > 0;

    // Build parameter list
    const params = [];
    if (hasPathParams) {
      params.push(...config.pathParams.map((param) => `${param}: string`));
    }
    if (hasQueryParams) {
      const queryParamTypes = Object.entries(config.queryParams)
        .map(([key, type]) => {
          const isOptional = type.endsWith("?");
          const baseType = isOptional ? type.slice(0, -1) : type;
          return `${key}${isOptional ? "?" : ""}: ${baseType}`;
        })
        .join(", ");

      // Check if all query params are optional
      const allOptional = Object.values(config.queryParams).every((type) =>
        type.endsWith("?"),
      );
      const optionalMarker = allOptional ? "?" : "";

      params.push(`queryParams${optionalMarker}: { ${queryParamTypes} }`);
    }
    if (hasRequest) {
      params.push(`request: ${requestName}`);
    }

    const requestValidation = hasRequest
      ? `    ${requestName}Schema.parse(request);`
      : "";

    // Build URL with path parameters
    let urlBuilder = `\`\${this.baseUrl}${config.path}\``;
    if (hasPathParams) {
      config.pathParams.forEach((param) => {
        urlBuilder = urlBuilder.replace(`{${param}}`, `\${${param}}`);
      });
    }

    // Add query parameters
    if (hasQueryParams) {
      urlBuilder += ` + (queryParams
        ? "?" +
          new URLSearchParams(queryParams as Record<string, string>).toString()
        : "")`;
    }

    const requestBody = hasRequest
      ? `      body: JSON.stringify(request),`
      : "";
    const responseType = hasResponse ? responseName : "void";
    const responseHandling = hasResponse
      ? `    const data = await response.json();\n    return ${responseName}Schema.parse(data);`
      : `    return;`;

    methods.push(`
  async ${endpointName}(${params.join(", ")}): Promise<${responseType}> {
${requestValidation}

    const url = ${urlBuilder};
    const response = await fetch(url, {
      method: '${config.method}',
      headers: {
        'Content-Type': 'application/json',
        ...this.defaultHeaders,
      },
${requestBody}
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new ApiError(\`HTTP \${response.status}: \${errorText}\`, response.status, errorText);
    }

${responseHandling}
  }`);
  });

  // Load template and replace placeholders
  const template = loadTemplate("api-client.template.ts");
  return template
    .replace("{{SCHEMAS}}", schemas.join("\n\n"))
    .replace("{{TYPES}}", types.join("\n"))
    .replace("{{METHODS}}", methods.join("\n"))
    .replace("{{BASE_URL}}", API_SPEC.baseUrl);
}

function generateTestHelpers() {
  const template = loadTemplate("test-helpers.template.ts");
  return template.replace("{{BASE_URL}}", API_SPEC.baseUrl);
}

function generatePackageJson() {
  return JSON.stringify(
    {
      name: "webauthn-api-client",
      version: "1.0.0",
      description: "TypeScript client for WebAuthn API",
      main: "dist/index.js",
      types: "dist/index.d.ts",
      scripts: {
        build: "tsc",
        test: "jest",
        "test:integration": "jest --testPathPattern=integration",
        "test:unit": "jest --testPathPattern=unit",
        "test:coverage": "jest --coverage",
        lint: "eslint src/**/*.ts",
        dev: "tsc --watch",
      },
      dependencies: {
        zod: "^3.22.0",
      },
      devDependencies: {
        "@types/jest": "^29.5.0",
        "@types/node": "^20.0.0",
        "@typescript-eslint/eslint-plugin": "^6.0.0",
        "@typescript-eslint/parser": "^6.0.0",
        eslint: "^8.0.0",
        jest: "^29.5.0",
        "ts-jest": "^29.1.0",
        typescript: "^5.0.0",
      },
      jest: {
        preset: "ts-jest",
        testEnvironment: "node",
        collectCoverageFrom: [
          "src/**/*.ts",
          "!src/**/*.test.ts",
          "!src/**/*.spec.ts",
        ],
        testMatch: ["**/tests/**/*.test.ts", "**/tests/**/*.spec.ts"],
      },
    },
    null,
    2,
  );
}

function generateIntegrationTest() {
  const template = loadTemplate("integration.test.template.ts");
  return template.replace(/{{BASE_URL}}/g, API_SPEC.baseUrl);
}

function generateAllRoutesTest() {
  const template = loadTemplate("all-routes.test.template.ts");
  return template.replace(/{{BASE_URL}}/g, API_SPEC.baseUrl);
}

function generateInviteCodesTest() {
  const template = loadTemplate("invite-codes.test.template.ts");
  return template.replace(/{{BASE_URL}}/g, API_SPEC.baseUrl);
}

function generateTestData() {
  const template = loadTemplate("test-data.template.ts");
  return template.replace(/{{BASE_URL}}/g, API_SPEC.baseUrl);
}

// Main execution
function main() {
  const outputDir = "../generated/ts-client";

  // Create directories
  if (!fs.existsSync(path.resolve(outputDir))) {
    fs.mkdirSync(path.resolve(outputDir), { recursive: true });
  }

  if (!fs.existsSync(path.resolve(outputDir, "src"))) {
    fs.mkdirSync(path.resolve(outputDir, "src"), { recursive: true });
  }

  if (!fs.existsSync(path.resolve(outputDir, "tests"))) {
    fs.mkdirSync(path.resolve(outputDir, "tests"), { recursive: true });
  }

  // Generate files
  fs.writeFileSync(
    path.resolve(outputDir, "src", "api-client.ts"),
    generateApiClient(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "src", "test-helpers.ts"),
    generateTestHelpers(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "package.json"),
    generatePackageJson(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "tests", "integration.test.ts"),
    generateIntegrationTest(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "tests", "all-routes.test.ts"),
    generateAllRoutesTest(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "tests", "invite-codes.test.ts"),
    generateInviteCodesTest(),
  );

  fs.writeFileSync(
    path.resolve(outputDir, "src", "test-data.ts"),
    generateTestData(),
  );

  // Generate TypeScript config
  const tsConfig = {
    compilerOptions: {
      target: "ES2020",
      module: "commonjs",
      lib: ["ES2020"],
      outDir: "./dist",
      rootDir: "./src",
      strict: true,
      esModuleInterop: true,
      skipLibCheck: true,
      forceConsistentCasingInFileNames: true,
      declaration: true,
      declarationMap: true,
      sourceMap: true,
    },
    include: ["src/**/*"],
    exclude: ["node_modules", "dist", "tests"],
  };

  fs.writeFileSync(
    path.resolve(outputDir, "tsconfig.json"),
    JSON.stringify(tsConfig, null, 2),
  );

  // Generate index file
  const indexContent = `export * from './api-client';
export * from './test-helpers';
export * from './test-data';
`;

  fs.writeFileSync(path.resolve(outputDir, "src", "index.ts"), indexContent);

  console.log("TypeScript client generated successfully!");
  console.log(`Output directory: ${outputDir}`);
  console.log("");
  console.log("To use the generated client:");
  console.log(`cd ${outputDir}`);
  console.log("npm install");
  console.log("npm run build");
  console.log("");
  console.log("To run integration tests:");
  console.log("npm run test:integration");
}

if (require.main === module) {
  main();
}
