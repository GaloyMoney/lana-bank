module.exports = {
  plugin: (_, documents) => {
    const imports = `import { getClient } from "../../client";`

    const functions = documents
      .filter((doc) => doc.document?.definitions?.length > 0)
      .map((doc) => {
        const operation = doc.document.definitions.find(
          (def) => def.kind === "OperationDefinition" && def.name
        );

        if (operation?.name?.value) {
          const name = operation.name.value;
          const camelCaseName = name.charAt(0).toLowerCase() + name.slice(1);
          return functionTemplate(camelCaseName, name);
        }
        return "";
      })
      .filter(Boolean)
      .join("\n");

    return [imports, functions].join("\n\n");
  },
};

const functionTemplate = (functionName, operationName) => {
  return `
export async function ${functionName}(variables: ${operationName}QueryVariables) {
  try {
    const response = await getClient().query<${operationName}Query, ${operationName}QueryVariables>({
      query: ${operationName}Document,
      variables,
    });
    return response;
  } catch (error) {
    if (error instanceof Error) {
      console.error("error in ${functionName}", error);
      return { error: error.message };
    }
    return { error: "An unknown error occurred" };
  }
}`;
};
