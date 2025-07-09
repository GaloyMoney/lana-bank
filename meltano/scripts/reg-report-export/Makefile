setup: requirements.txt
	# python -m venv env
	# source env/bin/activate
	pip install -r requirements.txt

run-export:
	DATASET_ID=dataform_${DATAFORM_SCHEMA_SUFFIX}  python3 main.py stg_01_npb4_17_persona_xml stg_02_npb4_17_referencia_xml

clean:
	rm ./*.xml
