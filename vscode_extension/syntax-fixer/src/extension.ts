// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';
import { writeFile } from 'fs';

async function runCommand(command: string): Promise<string | null> {
    try {
        const { stdout, stderr } = await promisify(exec)(command);
        return stdout;
    } catch (error) {
        return null;
    }
}


// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "syntax-fixer" is now active!');

	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	let disposable = vscode.commands.registerCommand('syntax-fixer.fixYourCode', async () => {
		// step 1: read all your code text
		let editor = vscode.window.activeTextEditor;
		if (editor) {
			let text = editor.document.getText();
			await promisify(writeFile)("/tmp/uwu.txt", text);
			let stdout = await runCommand(`cd /home/ritik/workspace/personal/syntax-fixer; cargo run /tmp/uwu.txt`);
			
			// step 3: rewrite all your code text
			editor.edit((editBuilder) => {
				let fullRange = new vscode.Range(editor!.document.positionAt(0), editor!.document.positionAt(editor!.document.getText().length));

				
				if (stdout) {
					editBuilder.replace(fullRange, stdout);
					vscode.window.showInformationMessage("success!");
				} else {
					vscode.window.showErrorMessage("had an issue fixing your bugs");

				}
			});
		}
	});

	context.subscriptions.push(disposable);
}

// This method is called when your extension is deactivated
export function deactivate() { }
