Building Documentation
======================

This project uses reStructuredText (RST) format for documentation, which can be built into various formats using Sphinx.

Prerequisites
-------------

Install Sphinx and the alabaster theme:

.. code-block:: bash

   pip install sphinx sphinx-autobuild

Building HTML Documentation
---------------------------

To build the HTML documentation:

.. code-block:: bash

   # Build documentation
   sphinx-build -b html . _build/html

   # Open in browser
   xdg-open _build/html/index.html  # Linux
   open _build/html/index.html      # macOS
   start _build/html/index.html     # Windows

Auto-rebuild During Development
--------------------------------

For automatic rebuilding while editing:

.. code-block:: bash

   sphinx-autobuild . _build/html

This will start a local web server at http://127.0.0.1:8000 and automatically rebuild when you save changes.

Building Other Formats
----------------------

PDF (requires LaTeX)
~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   sphinx-build -b latex . _build/latex
   cd _build/latex
   make

EPUB
~~~~

.. code-block:: bash

   sphinx-build -b epub . _build/epub

Man Pages
~~~~~~~~~

.. code-block:: bash

   sphinx-build -b man . _build/man

Documentation Structure
-----------------------

The documentation is organized as follows:

.. code-block:: text

   .
   ├── index.rst                    # Main documentation index
   ├── README.rst                   # User guide
   ├── QUICK_START.rst             # Quick start guide
   ├── PROJECT_STATUS.rst          # Project status
   ├── DEVELOPMENT_SUMMARY.rst     # Development history
   ├── conf.py                      # Sphinx configuration
   ├── examples/
   │   └── README.rst              # Configuration examples
   └── profiles/
       └── README.rst              # Battery profiles

Key Documentation Files
-----------------------

User Documentation
~~~~~~~~~~~~~~~~~~

- **README.rst**: Complete user guide with features, installation, and usage
- **QUICK_START.rst**: Quick start guide for immediate usage
- **examples/README.rst**: Example configuration files and usage patterns
- **profiles/README.rst**: Battery chemistry profiles and customization

Developer Documentation
~~~~~~~~~~~~~~~~~~~~~~~

- **PROJECT_STATUS.rst**: Current project status and feature checklist
- **DEVELOPMENT_SUMMARY.rst**: Complete development history with all commits
- **index.rst**: Documentation hub with navigation structure

Writing Documentation
---------------------

All documentation should be written in reStructuredText (RST) format. Here are some common patterns:

Headers
~~~~~~~

.. code-block:: rst

   Top Level Header
   ================

   Second Level
   ------------

   Third Level
   ~~~~~~~~~~~

Code Blocks
~~~~~~~~~~~

.. code-block:: rst

   .. code-block:: bash

      cargo build --release

   .. code-block:: python

      def hello():
          print("Hello")

Lists
~~~~~

.. code-block:: rst

   - Unordered item 1
   - Unordered item 2
     
     - Nested item

   1. Ordered item 1
   2. Ordered item 2

Tables
~~~~~~

.. code-block:: rst

   .. list-table::
      :header-rows: 1
      :widths: 25 75

      * - Column 1
        - Column 2
      * - Data 1
        - Data 2

Links
~~~~~

.. code-block:: rst

   External link: `GitHub <https://github.com>`_
   
   Internal link: :doc:`README`

Inline Markup
~~~~~~~~~~~~~

.. code-block:: rst

   **Bold text**
   *Italic text*
   ``code/literal``

Cleaning Build Artifacts
-------------------------

To remove build artifacts:

.. code-block:: bash

   rm -rf _build/

Troubleshooting
---------------

**Sphinx not found**
   Install Sphinx: ``pip install sphinx``

**Build warnings about missing references**
   Check that all referenced files exist and paths are correct

**Theme not rendering correctly**
   Ensure alabaster theme is installed: ``pip install alabaster``

For More Information
--------------------

- Sphinx documentation: https://www.sphinx-doc.org/
- reStructuredText primer: https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html
- RST cheat sheet: https://docutils.sourceforge.io/docs/user/rst/quickref.html
