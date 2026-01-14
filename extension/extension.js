import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

export default class GnomeShotExtension extends Extension {
    constructor(metadata) {
        super(metadata);
        this._settings = null;
        this._keybindings = [];
    }

    enable() {
        this._settings = this.getSettings();

        // Register keybindings
        this._addKeybinding('capture-shortcut', () => {
            this._launchGnomeShot();
        });

        console.log('GNOME Shot extension enabled');
    }

    disable() {
        // Remove all keybindings
        for (const name of this._keybindings) {
            Main.wm.removeKeybinding(name);
        }
        this._keybindings = [];
        this._settings = null;

        console.log('GNOME Shot extension disabled');
    }

    _addKeybinding(name, callback) {
        Main.wm.addKeybinding(
            name,
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            callback
        );
        this._keybindings.push(name);
    }

    _launchGnomeShot() {
        try {
            // Try to find gnome-shot in common locations
            const locations = [
                '/home/rb/Code/github/gnome-shot/target/debug/gnome-shot',
                '/home/rb/Code/github/gnome-shot/target/release/gnome-shot',
                '/usr/local/bin/gnome-shot',
                '/usr/bin/gnome-shot',
                GLib.build_filenamev([GLib.get_home_dir(), '.local', 'bin', 'gnome-shot']),
            ];

            let gnomeShotPath = null;
            for (const path of locations) {
                if (GLib.file_test(path, GLib.FileTest.IS_EXECUTABLE)) {
                    gnomeShotPath = path;
                    break;
                }
            }

            if (gnomeShotPath) {
                // Launch gnome-shot with --capture flag to auto-start capture
                const subprocess = Gio.Subprocess.new(
                    [gnomeShotPath, '--capture'],
                    Gio.SubprocessFlags.NONE
                );

                console.log('Launched gnome-shot from:', gnomeShotPath);
            } else {
                Main.notify('GNOME Shot', 'gnome-shot executable not found');
                console.error('gnome-shot executable not found');
            }
        } catch (e) {
            console.error('Failed to launch gnome-shot:', e);
            Main.notify('GNOME Shot', 'Failed to launch: ' + e.message);
        }
    }
}
