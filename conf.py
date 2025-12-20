# SPDX-License-Identifier: GPL-2.0-or-later
# Copyright (C) 2025 Marcus Folkesson

# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'DP832 Battery Simulator'
copyright = '2025, Marcus Folkesson'
author = 'Marcus Folkesson'
release = '1.0.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = []

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store', 'target', 'logs']

# Suppress warnings for missing images (screenshots may not be present yet)
suppress_warnings = ['image.not_readable']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'alabaster'
html_static_path = ['_static']

# Theme options
html_theme_options = {
    'description': 'Real-time battery simulator for Rigol DP832 power supply',
    'github_user': 'your-username',
    'github_repo': 'dp832-battery-sim',
    'github_banner': True,
    'github_button': True,
    'github_type': 'star',
}

# The master toctree document.
master_doc = 'index'

# The suffix(es) of source filenames.
source_suffix = '.rst'

# Pygments style for syntax highlighting
pygments_style = 'sphinx'

# If true, `todo` and `todoList` produce output, else they produce nothing.
todo_include_todos = False
