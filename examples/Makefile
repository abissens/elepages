deverver:
	cd output && devserver --reload

simple_md_1:
	elepages --source ./pages/1_mds --git-path ../ -vvv

simple_md_2:
	elepages --source ./pages/2_mds_with_metadata --git-path ../ -vvv

simple_md_3:
	elepages --source ./pages/3_mds_folders --git-path ../ -vvv

single_tpl:
	elepages --source ./pages/3_mds_folders --git-path ../ --handlebars-path ./templates/single -vvv

full_tpl:
	elepages --source ./pages/3_mds_folders --git-path ../ --handlebars ./templates/full -vvv

full_tpl_remote:
	elepages --source ./pages/3_mds_folders --git-path ../ --handlebars-remote  https://github.com/abissens/elepages.git --handlebars-path ./examples/templates/full -vvv
