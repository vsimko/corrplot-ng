use super::*;
    fn input() -> NormalizedMatrixV1 {
        NormalizedMatrixV1 {
            schema_version: 1, kind: "normalized_matrix".into(),
            labels: vec!["alpha".into(), "beta".into(), "gamma".into()],
            values: vec![vec![Some(1.0), Some(-0.5), None], vec![Some(-0.5), Some(1.0), None], vec![None, None, Some(1.0)]],
            source_indices: vec![0, 1, 2], semantics: MatrixSemantics::Correlation,
            display_type: DisplayType::Full, show_diagonal: true,
        }
    }

    #[test]
    fn scene_v1_to_scene_applies_label_typography_and_collision_policy() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace("\"alpha\"", "\"a-very-long-variable-label\"")
            .replace(
                "\"text\": \"a-very-long-variable-label\",",
                "\"text\": \"a-very-long-variable-label\", \"column_rotation\": 35,",
            )
            .replace(
                "\"glyph_method\": \"circle\",",
                "\"typography\": {\"label_font_family\": \"serif\", \"label_font_size\": 10, \"label_font_weight\": \"bold\", \"column_label_orientation\": \"vertical\", \"label_collision\": \"truncate\"},\n  \"glyph_method\": \"circle\",",
            );
        let scene = scene_v1_to_scene(&source, CellMethod::Circle).unwrap();
        let row_label = scene.texts.iter().find(|text| text.anchor == TextAnchor::End).unwrap();
        assert_eq!(row_label.font_family, "serif");
        assert_eq!(row_label.font_size, 10.0);
        assert_eq!(row_label.font_weight, "bold");
        assert!(row_label.text.ends_with('…'));
        assert!(row_label.text.chars().count() <= 12);
        let column_label = scene.texts.iter().find(|text| text.anchor == TextAnchor::Start && text.text.ends_with('…')).unwrap();
        assert_eq!(column_label.rotation_degrees, -35.0);

        let horizontal_source = source.replace("\"column_label_orientation\": \"vertical\"", "\"column_label_orientation\": \"horizontal\"");
        let horizontal = scene_v1_to_scene(&horizontal_source, CellMethod::Circle).unwrap();
        let horizontal_label = horizontal.texts.iter().find(|text| text.anchor == TextAnchor::Middle && text.text.ends_with('…')).unwrap();
        assert_eq!(horizontal_label.rotation_degrees, 0.0);

        use crate::output::{OutputOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        let svg = crate::raster::SceneSvgRenderer.render(
            &scene,
            &OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels)),
        ).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output"); };
        assert!(svg.contains("rotate(-35 146 60)"));

        let omit_source = source.replace("\"label_collision\": \"truncate\"", "\"label_collision\": \"omit\"");
        let omitted = scene_v1_to_scene(&omit_source, CellMethod::Circle).unwrap();
        assert!(omitted.texts.iter().find(|text| text.anchor == TextAnchor::End).unwrap().text.is_empty());
    }

    #[test]
    fn scene_v1_to_scene_wraps_long_labels_and_rejects_plotmath() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace("\"alpha\"", "\"a very long variable label\"")
            .replace(
                "\"glyph_method\": \"circle\",",
                "\"typography\": {\"label_font_family\": \"sans-serif\", \"label_font_size\": 10, \"label_font_weight\": \"normal\", \"column_label_orientation\": \"horizontal\", \"label_collision\": \"wrap\"},\n  \"glyph_method\": \"circle\",",
            );
        let scene = scene_v1_to_scene(&source, CellMethod::Circle).unwrap();
        let row_label = scene.texts.iter().find(|text| text.anchor == TextAnchor::End).unwrap();
        assert!(row_label.text.contains('\n'));
        assert!(row_label.text.lines().all(|line| line.chars().count() <= 12));
        use crate::output::{OutputOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        let svg = crate::raster::SceneSvgRenderer.render(
            &scene,
            &OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels)),
        ).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output"); };
        assert!(svg.contains("<tspan"));
        let rgba = crate::raster::RgbaRenderer.render(
            &scene,
            &OutputOptions::Rgba(crate::output::RasterOptions::new(100, 100)),
        ).unwrap();
        assert!(matches!(rgba, RenderOutput::Rgba(_)));

        let plotmath_source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace(
                "\"text\": \"alpha\",",
                "\"text\": \"alpha\", \"plotmath\": {\"prefix\": \"=\", \"source\": \"alpha\"},",
            );
        let error = parse_scene_v1_matrix(&plotmath_source).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_value");
        assert_eq!(error.path, "labels[0].plotmath");
    }

    #[test]
    fn scene_v1_to_scene_preserves_title_annotation_geometry_and_palette() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json");
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert_eq!((scene.width, scene.height), (380.0, 392.0));
        assert!(scene.texts.iter().any(|text| text.semantic_role.as_deref() == Some("document-title") && text.text == "Example correlation matrix"));
        assert_eq!(scene.description, "Synthetic correlations for alpha, beta, and gamma. Two pairwise values are unavailable.");
        use crate::output::{OutputOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        let svg = crate::raster::SceneSvgRenderer.render(
            &scene,
            &OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels)),
        ).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output"); };
        assert!(svg.contains("<desc>Synthetic correlations for alpha, beta, and gamma. Two pairwise values are unavailable.</desc>"));
        assert!(svg.contains("<title id=\"scene-title\">Example correlation matrix</title>"));
        assert!(svg.contains("aria-labelledby=\"scene-title\""));
        assert!(svg.contains("role=\"img\""));
        assert!(svg.contains("aria-label=\"Synthetic correlations for alpha, beta, and gamma. Two pairwise values are unavailable.\""));
        assert!(scene.texts.iter().any(|text| text.text == "Prototype annotation: synthetic example only"));
        assert_eq!((scene.cells[0].x, scene.cells[0].y, scene.cells[0].size), (104.0, 76.0, 84.0));
        let first_row_label = scene.texts.iter().find(|text| text.anchor == TextAnchor::End && text.text == "alpha").unwrap();
        assert_eq!((first_row_label.x, first_row_label.y), (92.0, 118.0));
        assert!(matches!(&scene.cells[0].glyph, Some(CellGlyph::Circle { radius, color })
            if *radius == 36.12 && *color == RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 }));
    }


    #[test]
    fn scene_v1_to_scene_preserves_typed_legend_and_cluster_regions() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace(
                "\"annotation\": \"Prototype annotation: synthetic example only\",",
                "\"annotation\": \"Prototype annotation: synthetic example only\",\n  \"legend\": [{\"kind\":\"missing\",\"label\":\"Not observed\",\"color\":\"#f2f2f2\"}],\n  \"cluster_regions\": [{\"id\":\"group-a\",\"variables\":[\"alpha\"],\"source_indices\":[0],\"row_start\":0,\"row_end\":0,\"column_start\":0,\"column_end\":0,\"x\":100,\"y\":72,\"width\":86,\"height\":86,\"color\":\"#2166ac\",\"line_width\":1.5}],"
            );
        let scene = scene_v1_to_scene(&source, CellMethod::Circle).unwrap();
        assert_eq!(scene.legend.len(), 1);
        assert_eq!(scene.legend[0].kind, "missing");
        assert_eq!(scene.legend[0].label, "Not observed");
        assert_eq!(scene.legend[0].color, RgbaColor { red: 242, green: 242, blue: 242, alpha: 255 });
        assert_eq!(scene.cluster_regions.len(), 1);
        let region = &scene.cluster_regions[0];
        assert_eq!(region.id, "group-a");
        assert_eq!(region.variables, vec!["alpha"]);
        assert_eq!(region.source_indices, vec![0]);
        assert_eq!((region.row_start, region.row_end, region.column_start, region.column_end), (0, 0, 0, 0));
        assert_eq!((region.x, region.y, region.width, region.height, region.line_width), (100.0, 72.0, 86.0, 86.0, 1.5));
        assert_eq!(region.color, RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 });
    }

    #[test]
    fn scene_v1_conversion_uses_explicit_cell_identity_not_array_order() {
        let source = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "significance":{"mode":"stars","thresholds":[0.05,0.01],"symbol":"*"},
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[
            {"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4},
            {"text":"b","row_x":1,"row_y":2,"column_x":3,"column_y":4}
          ],
          "cells":[
            {"row":1,"column":1,"value":1,"x":30,"y":30,"center_x":35,"center_y":35,"size":10,"encoding":{"kind":"circle","magnitude":1,"sign":1,"color":"#2166ac","radius":4}},
            {"row":0,"column":1,"value":-0.5,"x":20,"y":10,"center_x":25,"center_y":15,"size":10,"encoding":{"kind":"circle","magnitude":0.5,"sign":-1,"color":"#b2182b","radius":2}},
            {"row":1,"column":0,"value":-0.5,"x":10,"y":20,"center_x":15,"center_y":25,"size":10,"encoding":{"kind":"circle","magnitude":0.5,"sign":-1,"color":"#b2182b","radius":2}},
            {"row":0,"column":0,"value":1,"x":10,"y":10,"center_x":15,"center_y":15,"size":10,"encoding":{"kind":"circle","magnitude":1,"sign":1,"color":"#2166ac","radius":4}}
          ]
        }"##;
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert_eq!((scene.cells[0].x, scene.cells[0].y), (10.0, 10.0));
        assert_eq!((scene.cells[1].x, scene.cells[1].y), (20.0, 10.0));
        assert!(matches!(&scene.cells[1].glyph, Some(CellGlyph::Circle { color, radius })
            if color.red == 178 && color.green == 24 && color.blue == 43 && *radius == 2.0));
    }

    #[test]
    fn scene_v1_to_scene_honors_root_encoding_and_per_cell_glyph_methods() {
        let source = include_str!("../../../prototypes/shared-svg/scene-v1-construction.example.json");
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert!(matches!(&scene.cells[0].glyph, Some(CellGlyph::Square { .. })));
        assert!(matches!(&scene.cells[1].glyph, Some(CellGlyph::Pie { .. })));
        assert!(matches!(&scene.cells[2].glyph, Some(CellGlyph::Number { .. })));
    }

    #[test]
    fn scene_v1_to_scene_lays_out_figure_text_lines() {
        let source = include_str!("../../../prototypes/shared-svg/scene-v1-construction.example.json");
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert!(scene.texts.iter().any(|text| text.text == "A caption"
            && text.x == 50.0 && text.y == 90.0 && text.font_size == 10.0
            && text.anchor == TextAnchor::Middle
            && text.font_family == "serif" && text.font_weight == "bold" && text.font_style == "italic"
            && text.semantic_role.as_deref() == Some("caption")));
        assert!(scene.texts.iter().any(|text| text.text == "second line"
            && text.x == 50.0 && text.y == 102.0 && text.anchor == TextAnchor::Middle
            && text.font_family == "serif" && text.font_weight == "bold" && text.font_style == "italic"
            && text.semantic_role.as_deref() == Some("caption")));

        use crate::output::{OutputOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        let svg = crate::raster::SceneSvgRenderer.render(
            &scene,
            &OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels)),
        ).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output"); };
        assert!(svg.contains("data-semantic-role=\"caption\""));
        assert!(svg.contains("aria-label=\"caption: A caption\""));
    }

    #[test]
    fn scene_v1_to_scene_preserves_cell_text_annotations() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace("\"text_annotations\": []", "\"text_annotations\": [{\"row\":1,\"column\":2,\"role\":\"coefficient\",\"text\":\"0.65\",\"color\":\"black\",\"font_size\":9,\"font_weight\":\"bold\",\"font_style\":\"normal\"}]");
        let scene = scene_v1_to_scene(&source, CellMethod::Circle).unwrap();
        assert!(scene.texts.iter().any(|text| {
            text.text == "0.65" && text.x == 314.0 && text.y == 202.0
                && text.font_size == 9.0 && text.font_weight == "bold" && text.font_style == "normal"
                && text.color.red == 0 && text.color.green == 0
                && text.color.blue == 0 && text.color.alpha == 255
                && text.anchor == TextAnchor::Middle
        }));
    }

    #[test]
    fn scene_v1_rejects_text_annotations_outside_matrix() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace("\"text_annotations\": []", "\"text_annotations\": [{\"row\":3,\"column\":0,\"role\":\"value\",\"text\":\"x\",\"color\":\"black\",\"font_size\":9,\"font_weight\":\"normal\",\"font_style\":\"normal\"}]");
        let error = scene_v1_to_scene(&source, CellMethod::Circle).unwrap_err();
        assert_eq!(error.code, "input.scene.cell_out_of_bounds");
        assert_eq!(error.path, "text_annotations[0].row");
    }

    #[test]
    fn builds_deterministic_scene_and_preserves_missing_values() {
        let scene = matrix_to_scene_v1(&input(), &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.len(), 9);
        assert_eq!(scene.cells[0].glyph, Some(CellGlyph::Circle { radius: CELL_SIZE * 0.43, color: RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 } }));
        assert!(scene.texts.iter().any(|text| text.text == "NA"));
        assert!(scene.texts.iter().any(|text| text.text == "alpha"));
        assert_eq!(scene.width, LEFT_MARGIN + 3.0 * (CELL_SIZE + CELL_GAP) + RIGHT_MARGIN);
    }

    #[test]
    fn supports_all_seven_declared_cell_methods() {
        for method in [CellMethod::Circle, CellMethod::Square, CellMethod::Ellipse, CellMethod::Number, CellMethod::Shade, CellMethod::Color, CellMethod::Pie] {
            let options = SceneBuildOptions { cell_method: method, ..Default::default() };
            let scene = matrix_to_scene_v1(&input(), &options).unwrap();
            assert_eq!(scene.cells.len(), 9);
            if method == CellMethod::Ellipse {
                assert!(matches!(&scene.cells[0].glyph, Some(CellGlyph::Ellipse { rotation_degrees, .. }) if *rotation_degrees == -45.0));
                assert!(matches!(&scene.cells[1].glyph, Some(CellGlyph::Ellipse { rotation_degrees, .. }) if *rotation_degrees == 45.0));
            }
            scene.validate().unwrap();
        }
    }

    #[test]
    fn display_type_masks_cells_without_removing_or_renumbering_them() {
        for (display_type, expected_visible) in [
            (DisplayType::Full, vec![true; 9]),
            (DisplayType::Upper, vec![true, true, true, false, true, true, false, false, true]),
            (DisplayType::Lower, vec![true, false, false, true, true, false, true, true, true]),
            (DisplayType::Diagonal, vec![true, false, false, false, true, false, false, false, true]),
        ] {
            let mut matrix = input();
            matrix.display_type = display_type;
            let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
            assert_eq!(scene.cells.len(), 9);
            assert_eq!(scene.cells.iter().map(|cell| cell.visible).collect::<Vec<_>>(), expected_visible);
        }
    }

    #[test]
    fn show_diagonal_hides_diagonal_independently_of_triangle_mode() {
        let mut matrix = input();
        matrix.display_type = DisplayType::Upper;
        matrix.show_diagonal = false;
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.iter().map(|cell| cell.visible).collect::<Vec<_>>(),
            vec![false, true, true, false, false, true, false, false, false]);
    }

    #[test]
    fn missing_annotations_follow_the_display_mask() {
        let mut matrix = input();
        matrix.display_type = DisplayType::Upper;
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        let missing: Vec<_> = scene.texts.iter().filter(|text| text.text == "NA").collect();
        assert_eq!(missing.len(), 2);
        let expected_x = LEFT_MARGIN + 2.0 * (CELL_SIZE + CELL_GAP) + CELL_SIZE / 2.0;
        let expected_positions = vec![
            (expected_x, TOP_MARGIN + CELL_SIZE / 2.0 - 5.0),
            (expected_x, TOP_MARGIN + (CELL_SIZE + CELL_GAP) + CELL_SIZE / 2.0 - 5.0),
        ];
        let actual_positions: Vec<_> = missing.iter().map(|text| (text.x, text.y)).collect();
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn general_numeric_values_use_a_data_derived_magnitude() {
        let mut matrix = input();
        matrix.semantics = MatrixSemantics::GeneralNumeric;
        matrix.labels = vec!["alpha".into(), "beta".into()];
        matrix.source_indices = vec![0, 1];
        matrix.values = vec![vec![Some(0.0), Some(5.0)], vec![Some(-10.0), Some(2.0)]];
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells[2].glyph, Some(CellGlyph::Circle { radius: CELL_SIZE * 0.43, color: RgbaColor { red: 178, green: 24, blue: 43, alpha: 255 } }));
    }

    #[test]
    fn built_scene_is_consumed_by_both_existing_renderers() {
        use crate::output::{OutputOptions, RasterOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        use crate::raster::{RgbaRenderer, SceneSvgRenderer};
        let scene = matrix_to_scene_v1(&input(), &SceneBuildOptions::default()).unwrap();
        let svg_options = OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels));
        let svg = SceneSvgRenderer.render(&scene, &svg_options).unwrap();
        assert!(matches!(svg, RenderOutput::Svg(_)));
        let rgba_options = OutputOptions::Rgba(RasterOptions::new(320, 320));
        let rgba = RgbaRenderer.render(&scene, &rgba_options).unwrap();
        assert!(matches!(rgba, RenderOutput::Rgba(_)));
    }

    #[test]
    fn rejects_duplicate_source_indices_with_stable_path() {
        let mut matrix = input();
        matrix.source_indices = vec![0, 0, 2];
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.code, "input.matrix.invalid_source_indices");
        assert_eq!(error.path, "source_indices[1]");
    }

    #[test]
    fn rejects_out_of_range_source_indices() {
        let mut matrix = input();
        matrix.source_indices = vec![0, 1, 3];
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.code, "input.matrix.invalid_source_indices");
        assert_eq!(error.path, "source_indices[2]");
    }

    #[test]
    fn rejects_overflowing_json_numbers_as_invalid_json() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace(r#""width": 380"#, r#""width": 1e999"#);
        let error = parse_scene_v1_matrix(&source).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_json");
        assert_eq!(error.path, "$");
    }

    #[test]
    fn maps_cell_validation_failures_to_stable_matrix_paths_and_coordinates() {
        let mut matrix = input();
        matrix.values[0][1] = Some(1.25);
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.code, "input.matrix.out_of_range");
        assert_eq!(error.path, "values[0][1]");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(1));

        matrix.values[0][1] = Some(f64::NAN);
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.path, "values[0][1]");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(1));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut matrix = input();
        matrix.schema_version = 2;
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.path, "schema_version");
    }

    #[test]
    fn validates_optional_scene_v1_typography_legend_and_display_metadata() {
        let base = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let valid = base.replace("\"title\":\"\"", "\"title\":\"\",\"typography\":{\"label_font_family\":\"sans\",\"label_font_size\":12,\"label_font_weight\":\"bold\",\"column_label_orientation\":\"vertical\",\"label_collision\":\"wrap\"},\"legend\":[{\"kind\":\"missing\",\"label\":\"NA\",\"color\":\"#f2f2f2\"}],\"glyph_method\":\"circle\",\"display_type\":\"full\",\"show_diagonal\":true");
        assert!(parse_scene_v1_matrix(&valid).is_ok());
        let invalid = valid.replace("\"label_font_size\":12", "\"label_font_size\":60");
        let error = parse_scene_v1_matrix(&invalid).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_value");
        assert_eq!(error.path, "typography.label_font_size");
        let invalid_legend = valid.replace("\"#f2f2f2\"}],\"glyph_method\"", "\"red\"}],\"glyph_method\"");
        let error = parse_scene_v1_matrix(&invalid_legend).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_color");
        assert_eq!(error.path, "legend[0].color");
    }

    #[test]
    fn validates_scene_v1_cluster_regions_figure_text_global_significance_and_annotations() {
        let base = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let valid = base.replace(
            "\"title\":\"\"",
            r###""title":"","cluster_regions":[{"id":"c1","variables":["a"],"source_indices":[0],"row_start":0,"row_end":0,"column_start":0,"column_end":0,"x":1,"y":2,"width":10,"height":10,"color":"#2166ac","line_width":1}],"figure_text":[{"role":"caption","text":"caption","lines":["caption"],"alignment":"center","font_family":"sans","font_size":10,"font_weight":"normal","font_style":"italic","line_height":1.2,"spacing_after":0,"x":0,"y":90}],"significance":{"mode":"stars","thresholds":[0.05,0.01],"symbol":"*"},"text_annotations":[{"row":0,"column":0,"role":"coefficient","text":"1.0","color":"black","font_size":8,"font_weight":"bold","font_style":"normal"}]"###
        );
        assert!(parse_scene_v1_matrix(&valid).is_ok());

        let unknown_variable = valid.replace(r#""variables":["a"],"#, r#""variables":["missing"],"#);
        let error = parse_scene_v1_matrix(&unknown_variable).unwrap_err();
        assert_eq!(error.path, "cluster_regions[0].variables[0]");

        let out_of_range_source = valid.replace(r#""source_indices":[0]"#, r#""source_indices":[1]"#);
        let error = parse_scene_v1_matrix(&out_of_range_source).unwrap_err();
        assert_eq!(error.path, "cluster_regions[0].source_indices[0]");

        let out_of_range_bounds = valid.replace(r#""row_end":0"#, r#""row_end":1"#);
        let error = parse_scene_v1_matrix(&out_of_range_bounds).unwrap_err();
        assert_eq!(error.path, "cluster_regions[0].row_end");

        let reversed_bounds = valid.replace(r#""row_start":0"#, r#""row_start":1"#);
        let error = parse_scene_v1_matrix(&reversed_bounds).unwrap_err();
        assert_eq!(error.path, "cluster_regions[0].row_start");

        for (bad, expected_path) in [
            (valid.replace("\"line_width\":1", "\"line_width\":0"), "cluster_regions[0].line_width"),
            (valid.replace("\"font_size\":10", "\"font_size\":50"), "figure_text[0].font_size"),
            (valid.replace("\"thresholds\":[0.05,0.01]", "\"thresholds\":[1.5]"), "significance.thresholds[0]"),
            (valid.replace("\"font_size\":8", "\"font_size\":0"), "text_annotations[0].font_size"),
        ] {
            let error = parse_scene_v1_matrix(&bad).unwrap_err();
            assert_eq!(error.path, expected_path);
        }
    }

    #[test]
    fn validates_scene_v1_cell_encoding_significance_and_confidence_interval() {
        let base = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1,
            "encoding":{"kind":"circle","magnitude":1,"sign":1,"color":"#2166ac","fill_opacity":0.5},
            "significance":{"p_value":0.01,"status":"significant","annotation":"*","blank":false},
            "confidence_interval":{"lower":0.2,"upper":0.8,"x1":1,"x2":2,"y":1,"cap_height":1,"stroke_width":1,"color":"#2166ac"}}]
        }"##;
        assert!(parse_scene_v1_matrix(base).is_ok());
        let bad_encoding = base.replace("\"magnitude\":1", "\"magnitude\":2");
        let error = parse_scene_v1_matrix(&bad_encoding).unwrap_err();
        assert_eq!(error.path, "cells[0].encoding.magnitude");
        let bad_significance = base.replace("\"status\":\"significant\"", "\"status\":\"maybe\"");
        let error = parse_scene_v1_matrix(&bad_significance).unwrap_err();
        assert_eq!(error.path, "cells[0].significance.status");
        let bad_interval = base.replace("\"lower\":0.2", "\"lower\":0.9");
        let error = parse_scene_v1_matrix(&bad_interval).unwrap_err();
        assert_eq!(error.path, "cells[0].confidence_interval.lower");
    }

    #[test]
    fn parses_scene_v1_by_explicit_cell_identity_and_preserves_nulls() {
        let source = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[
            {"text":"alpha","row_x":1,"row_y":2,"column_x":3,"column_y":4},
            {"text":"beta","row_x":1,"row_y":2,"column_x":3,"column_y":4}
          ],
          "cells":[
            {"row":1,"column":1,"value":1,"x":10,"y":10,"center_x":15,"center_y":15,"size":10},
            {"row":0,"column":1,"value":null,"x":20,"y":10,"center_x":25,"center_y":15,"size":10},
            {"row":1,"column":0,"value":null,"x":10,"y":20,"center_x":15,"center_y":25,"size":10},
            {"row":0,"column":0,"value":1,"x":10,"y":10,"center_x":15,"center_y":15,"size":10}
          ]
        }"##;
        let matrix = parse_scene_v1_matrix(source).unwrap();
        assert_eq!(matrix.labels, vec!["alpha", "beta"]);
        assert_eq!(matrix.values, vec![vec![Some(1.0), None], vec![None, Some(1.0)]]);
        assert_eq!(matrix.source_indices, vec![0, 1]);
        assert_eq!(matrix.semantics, MatrixSemantics::Correlation);
    }

    #[test]
    fn scene_conversion_preserves_cell_significance_and_confidence_interval_metadata() {
        let source = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "significance":{"mode":"stars","thresholds":[0.05,0.01],"symbol":"*"},
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1.0,"x":10,"y":20,"center_x":15,"center_y":25,"size":10,
            "p_value":0.02,"normalized_value":0.75,
            "significance":{"p_value":0.02,"status":"significant","annotation":"*","blank":false},
            "confidence_interval":{"lower":0.8,"upper":1.0,"x1":11,"x2":19,"y":24,"cap_height":2,"stroke_width":1.5,"color":"#2166ac"}}]
        }"##;
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert_eq!(scene.significance, Some(SceneSignificancePolicy {
            mode: "stars".into(), thresholds: vec![0.05, 0.01], symbol: "*".into(),
        }));
        let cell = &scene.cells[0];
        assert_eq!(cell.p_value, Some(0.02));
        assert_eq!(cell.normalized_value, Some(0.75));
        assert_eq!(cell.significance, Some(SceneCellSignificance {
            p_value: Some(0.02), status: "significant".into(), annotation: Some("*".into()), blank: false,
        }));
        assert_eq!(cell.confidence_interval, Some(SceneConfidenceInterval {
            lower: 0.8, upper: 1.0, x1: 11.0, x2: 19.0, y: 24.0,
            cap_height: 2.0, stroke_width: 1.5,
            color: RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 },
        }));
        assert_eq!((cell.x, cell.y, cell.size), (10.0, 20.0, 10.0));
    }

    #[test]
    fn parses_the_shared_canonical_scene_v1_fixture() {
        let matrix = parse_scene_v1_matrix(include_str!("../../../fixtures/scene-v1/scene-v1.example.json")).unwrap();
        assert_eq!(matrix.labels, vec!["alpha", "beta", "gamma"]);
        assert_eq!(matrix.values[0], vec![Some(1.0), Some(0.65), Some(-0.4)]);
        assert_eq!(matrix.values[1][2], None);
        assert_eq!(matrix.source_indices, vec![0, 1, 2]);
        assert_eq!(matrix.semantics, MatrixSemantics::Correlation);
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.len(), 9);
        scene.validate().unwrap();
    }

    #[test]
    fn parser_preserves_triangle_display_options_for_scene_construction() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1.example.json")
            .replace(
                "\"glyph_method\": \"circle\"",
                "\"glyph_method\": \"circle\", \"display_type\": \"upper\", \"show_diagonal\": false",
            );
        let matrix = parse_scene_v1_matrix(&source).unwrap();
        assert_eq!(matrix.display_type, DisplayType::Upper);
        assert!(!matrix.show_diagonal);
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.iter().map(|cell| cell.visible).collect::<Vec<_>>(),
            vec![false, true, true, false, false, true, false, false, false]);
    }

    #[test]
    fn scene_v1_parser_rejects_unsupported_versions_and_duplicate_cell_identity() {
        let source = r##"{
          "schema_version":2,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let error = parse_scene_v1_matrix(source).unwrap_err();
        assert_eq!(error.code, "input.scene.unsupported_version");
        assert_eq!(error.path, "schema_version");

        let duplicate = source.replace("\"schema_version\":2", "\"schema_version\":1")
            .replace("\"labels\":[{\"text\":\"a\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4}]",
                     "\"labels\":[{\"text\":\"a\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4},{\"text\":\"b\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4}]")
            .replace("\"cells\":[{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1}]",
                     "\"cells\":[{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":0,\"column\":1,\"value\":0,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":1,\"column\":0,\"value\":0,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1}]");
        let error = parse_scene_v1_matrix(&duplicate).unwrap_err();
        assert_eq!(error.code, "input.scene.duplicate_cell");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(0));
    }

    #[test]
    fn scene_v1_parser_rejects_invalid_json_and_general_semantics_mismatch() {
        let error = parse_scene_v1_matrix("{").unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_json");

        let source = r##"{
          "schema_version":1,"kind":"general_matrix_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "matrix_semantics":{"mode":"correlation"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":2,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let error = parse_scene_v1_matrix(source).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_semantics");
        assert_eq!(error.path, "matrix_semantics.mode");

        let valid_general = source.replace(
            "\"mode\":\"correlation\"",
            "\"mode\":\"general_numeric\",\"color_limits\":[0,2],\"color_mapping\":\"sequential\",\"normalization\":{\"intercept\":0,\"zoom\":1}",
        );
        let matrix = parse_scene_v1_matrix(&valid_general).unwrap();
        assert_eq!(matrix.semantics, MatrixSemantics::GeneralNumeric);
        assert_eq!(matrix.values, vec![vec![Some(2.0)]]);

        let missing_limits = valid_general.replace(",\"color_limits\":[0,2]", "");
        let error = parse_scene_v1_matrix(&missing_limits).unwrap_err();
        assert_eq!(error.code, "input.scene.missing_field");
        assert_eq!(error.path, "matrix_semantics.color_limits");

        let unknown_field = valid_general.replace("\"title\":\"\"", "\"title\":\"\",\"unexpected\":true");
        let error = parse_scene_v1_matrix(&unknown_field).unwrap_err();
        assert_eq!(error.code, "input.scene.unknown_field");
        assert_eq!(error.path, "$.unexpected");
    }

    #[test]
    fn scene_v1_parser_rejects_invalid_display_options_instead_of_defaulting() {
        let source = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;

        let invalid_display_type = source.replace(
            "\"title\":\"\"",
            "\"title\":\"\",\"display_type\":\"sideways\"",
        );
        let error = parse_scene_v1_matrix(&invalid_display_type).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_value");
        assert_eq!(error.path, "display_type");

        let invalid_display_type_type = source.replace(
            "\"title\":\"\"",
            "\"title\":\"\",\"display_type\":true",
        );
        let error = parse_scene_v1_matrix(&invalid_display_type_type).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_type");
        assert_eq!(error.path, "display_type");

        let invalid_diagonal_type = source.replace(
            "\"title\":\"\"",
            "\"title\":\"\",\"show_diagonal\":\"yes\"",
        );
        let error = parse_scene_v1_matrix(&invalid_diagonal_type).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_type");
        assert_eq!(error.path, "show_diagonal");

        let valid_options = source.replace(
            "\"title\":\"\"",
            "\"title\":\"\",\"display_type\":\"upper\",\"show_diagonal\":false",
        );
        let matrix = parse_scene_v1_matrix(&valid_options).unwrap();
        assert_eq!(matrix.display_type, DisplayType::Upper);
        assert!(!matrix.show_diagonal);
    }


    #[test]
    fn scene_v1_statistical_overlays_are_retained() {
        let source = include_str!("../../../fixtures/scene-v1/scene-v1-statistical-overlays.example.json");
        let scene = scene_v1_to_scene(source, CellMethod::Circle).unwrap();
        assert_eq!(scene.significance.as_ref().unwrap().mode, "stars");
        assert_eq!(scene.cells[0].p_value, Some(0.001));
        assert_eq!(scene.cells[0].significance.as_ref().unwrap().annotation.as_deref(), Some("**"));
        assert!(scene.cells[0].confidence_interval.is_some());
        assert_eq!(scene.cells[1].significance.as_ref().unwrap().status, "not_significant");
        assert_eq!(scene.cells[5].significance.as_ref().unwrap().status, "unknown");
        assert_eq!(scene.cells[6].significance.as_ref().unwrap().blank, true);
        assert!(scene.cells[6].confidence_interval.is_some());
        assert!(!scene.cells[8].visible);
        assert_eq!(scene.cells[8].significance.as_ref().unwrap().annotation.as_deref(), Some("HIDDEN"));
    }

