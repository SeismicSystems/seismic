# Initialize the CLI subdirectory

Now that the `contracts` subdirectory has been initialized, you should now initialize the `cli` subdirectory that will be used to interact with the deployed contracts.

1. **Navigate to the contracts subdirectory:**

```bash
# Assuming you are currently in the contracts directory
cd ../cli
```

2. **Initialize a new bun project**

```bash
bun init -y
```

3. Now, create an `src/` folder and move `index.ts` there.

```bash
mkdir -p src && mv -t src index.ts
```

4. Now, edit `package.json` to be the following:

```json
{
    "name": "walnut-cli",
    "license": "MIT License",
    "type": "module",
    "scripts": {
        "dev": "bun run src/index.ts"
    },
    "dependencies": {
        "dotenv": "^16.4.7",
        "seismic-viem": "1.0.9",
        "viem": "^2.22.3"
    },
    "devDependencies": {
        "@types/node": "^22.7.6",
        "typescript": "^5.6.3"
    }
}
```

5. Edit `.gitignore` to be:

```gitignore
node_modules
```

Your environment is now set!
