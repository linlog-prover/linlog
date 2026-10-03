# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The exports compile and render: every document the core tests
# pin in core/tests/snapshots (the fragments inside a document), and a
# proof in each format as the CLI writes it, with pdfLaTeX from a minimal
# TeX Live, Typst with curryst from nixpkgs and resvg, offline; the CLI's
# PNG and PDF are checked with pngcheck, poppler and veraPDF. The curryst here is the version
# `linlog::export::typst::CURRYST' names; they change together. The LaTeX
# and Typst output names no font, so Typst sees only the fonts it embeds;
# resvg sees Euler Math and no system font. Anything either prints fails
# the check, so a font they cannot find is not hidden by a fallback.
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      snapshots = lib.fileset.toSource {
        root = ../core/tests/snapshots;
        fileset = ../core/tests/snapshots;
      };

      latex = pkgs.texliveBasic.withPackages (ps: [
        ps.amsfonts
        ps.cmll
        ps.ebproof
        ps.standalone
      ]);

      typst = pkgs.typst.withPackages (ps: [ ps.curryst_0_6_0 ]);

      eulerMath = "${pkgs.texlivePackages.euler-math.tex}/fonts/opentype/public/euler-math";
    in
    {
      checks.export =
        pkgs.runCommand "check-export"
          {
            nativeBuildInputs = [
              config.packages.linlog-cli
              latex
              typst
              pkgs.resvg
              pkgs.pngcheck
              pkgs.poppler-utils
              pkgs.verapdf
            ];
          }
          ''
            export HOME=$TMPDIR
            cp ${snapshots}/* .
            # A fragment goes into a document of the packages it names.
            for file in *.frag.tex; do
              {
                printf '\\documentclass{article}\n\\usepackage{amssymb}\n'
                printf '\\usepackage{cmll}\n\\usepackage{ebproof}\n\\begin{document}\n'
                cat "$file"
                printf '\n\\end{document}\n'
              } >"''${file%.frag.tex}-fragment.tex"
              rm "$file"
            done
            for file in *.frag.typ; do
              {
                printf '#import "@preview/curryst:0.6.0": prooftree, rule\n'
                printf '#set page(width: auto, height: auto, margin: 5pt)\n'
                cat "$file"
              } >"''${file%.frag.typ}-fragment.typ"
              rm "$file"
            done
            linlog prove -i --format latex --standalone --output cli.tex '!A, A -o B |- B * !A'
            linlog prove --format typst --standalone --output cli.typ 'A & B, !C |- (B + A) * !C'
            linlog prove -i --format svg --output cli.svg '!A, A -o B |- B * !A'
            linlog prove --net --output cli-net.svg 'A -o B, B -o C |- A -o C'
            # The command's PNG and PDF, rendered with the font it embeds: a
            # valid image, and a PDF whose text is text.
            linlog prove -i --output cli.png '!A, A -o B |- B * !A' 2>/dev/null
            pngcheck -q cli.png
            linlog prove --net --output cli-net.pdf 'A -o B, B -o C |- A -o C' 2>/dev/null
            linlog prove -i --output cli.pdf '!A, A -o B |- B * !A' 2>/dev/null
            pdffonts cli.pdf | grep -q Euler-Math
            pdftotext cli.pdf - | grep -q '⊢'
            # Every profile conforms, by veraPDF: PDF/A-4 by default,
            # PDF/A-2u compatible, PDF/A-2a and PDF/UA-1 accessible. The
            # date is the build's SOURCE_DATE_EPOCH.
            linlog prove -i --style compatible=true --output cli-2u.pdf '!A, A -o B |- B * !A' 2>/dev/null
            linlog prove -i --style accessible=true --output cli-ua.pdf '!A, A -o B |- B * !A' 2>/dev/null
            for check in "cli.pdf 4" "cli-net.pdf 4" "cli-2u.pdf 2u" "cli-ua.pdf 2a" "cli-ua.pdf ua1"; do
              set -- $check
              verapdf --format text --flavour "$2" "$1" >verdict 2>&1
              grep -q '^PASS' verdict || { cat verdict; exit 1; }
            done
            for file in *.tex; do
              pdflatex -interaction=nonstopmode -halt-on-error "$file" >/dev/null ||
                { cat "''${file%.tex}.log"; exit 1; }
            done
            for file in *.typ; do
              typst compile --ignore-system-fonts "$file" 2>log
              if [ -s log ]; then cat log; exit 1; fi
            done
            for file in *.svg; do
              resvg --skip-system-fonts --use-fonts-dir ${eulerMath} "$file" "''${file%.svg}.png" 2>log
              if [ -s log ]; then cat log; exit 1; fi
            done
            touch $out
          '';
    };
}
