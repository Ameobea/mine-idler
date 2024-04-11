<script lang="ts">
  import * as d3 from 'd3';

  export let totalQuality: number;
  export let buckets: number[];

  const width = 180;
  const height = 60;
  const margin = { top: 0, right: 0, bottom: 0, left: 0 };
  const barWidth = (width - margin.left - margin.right) / buckets.length;

  const genLabel = (bucketIx: number) => {
    const count = buckets[bucketIx];
    const bucketStart = bucketIx / buckets.length;
    const bucketEnd = (bucketIx + 1) / buckets.length;

    return `${bucketStart.toFixed(2)}-${bucketEnd.toFixed(2)}: ${count}`;
  };

  const x = d3
    .scaleLinear()
    .domain([0, 32])
    .range([margin.left, width - margin.right]);
  const y = d3
    .scaleLinear()
    .domain([0, d3.max(buckets) ?? 0])
    .range([height - margin.bottom, margin.top]);

  let container: HTMLDivElement;

  $: if (container) {
    container.innerHTML = '';

    const svg = d3.select(container).append('svg').attr('width', width).attr('height', height);

    svg
      .style('border', '1px solid #cccccc22')
      .selectAll('rect')
      .data(buckets)
      .join('rect')
      .attr('x', (_d, i) => x(i))
      .attr('y', (d) => y(d))
      .attr('bucket-ix', (_d, i) => i)
      .attr('width', barWidth)
      .attr('height', (d) => height - margin.bottom - y(d))
      .attr('fill', 'steelblue');

    // labels on hover
    svg
      .selectAll('rect')
      .on('mouseover', (e) => {
        const rect = e.target as SVGRectElement;
        const bucketIx = +rect.getAttribute('bucket-ix')!;
        const label = genLabel(bucketIx);

        svg
          .append('text')
          .attr('x', x(bucketIx) + barWidth / 2)
          .attr('y', y(buckets[bucketIx]) - 4)
          .attr('text-anchor', 'middle')
          .attr('dominant-baseline', 'middle')
          .text(label)
          .attr('fill', 'white')
          .attr('font-size', '10px');
      })
      .on('mouseout', () => {
        svg.selectAll('text').remove();
      });
  }
</script>

<div bind:this={container} class="quality-histogram"></div>

<style>
  .quality-histogram {
    display: block;
    margin: 0 auto;
  }
</style>
