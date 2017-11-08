
// Kaikenlaisia funktioita äänestykseen
// Tarvii jQueryn




function vote(votingId) {
	
	var form = $('#voting-form-' + votingId);
	var selectedCandidateId = form.find('input[name=candidate]:checked').val();
	
	if (selectedCandidateId == null) {
		return;
	}
	
	var confirmation = confirm('Haluatko varmasti antaa äänesi?');
	
	if (!confirmation) {
		return;
	}
	
	form.find('input, button').prop('disabled', true);
	
	var query = $.post(votingId + '/vote/',
		{ candidate : selectedCandidateId }
	).done(function(data) {
	}).fail(function(data) {
		alert('Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!');
	});
	
	refreshVotingList();
	
}

function refreshVotingList() {
	
	var votingArea = $('#voting-list-area');
	
	var query = $.get('votings/', function(data) {
		votingArea.html(data);
	})
	.fail(function() {
		alert('Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä.');
	});
	
}

$(document).ready(function() {
	refreshVotingList();	
});