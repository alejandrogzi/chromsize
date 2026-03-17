process CHROMSIZE {
    tag "$genome"
    label 'process_low'

    conda "${moduleDir}/environment.yml"
    container "${ workflow.containerEngine == 'singularity' && !task.ext.singularity_pull_docker_container ?
        '' : 
        'ghcr.io/alejandrogzi/chromsize:latest' }"

    input:
    tuple val(meta), path(genome)

    output:
    tuple val(meta), path("${genome.baseName}/chrom.sizes"), emit: chromsize
    path "versions.yml"           , emit: versions

    when:
    task.ext.when == null || task.ext.when

    script:
    def args = task.ext.args ?: ''
    def prefix = task.ext.prefix ?: genome.baseName
    """
    chromsize \\
        $args \\
        -s $genome \\
        -o ${prefix}
        
    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        chromsize: \$(chromsize --version | sed -e "s/chromsize v//g")
    END_VERSIONS
    """

    stub:
    def prefix = task.ext.prefix ?: "${meta.id}"
    """
    touch ${prefix}
    touch ${prefix}/chrom.sizes

    cat <<-END_VERSIONS > versions.yml
    "${task.process}":
        chromsize: \$(chromsize --version | sed -e "s/chromsize v//g")
    END_VERSIONS
    """
}
